//! What the binary is told by its environment (T-09).
//!
//! [`Config::from_vars`] is a pure function of a variable lookup, so every case
//! is a test (`tests/smoke_config.rs`); [`Config::from_env`] is the one place the
//! process environment is read. Three names:
//!
//! - `PORT` — set by Fly. Present: bind `0.0.0.0:<PORT>`. Absent: bind
//!   `127.0.0.1:3000`, the local default.
//! - `POPQUIZ_PUBLIC_URL` — the scheme and host people reach the room at,
//!   e.g. `https://rustnyc-popquiz.fly.dev`. `join_url` is `{this}/{code}` and
//!   the host's resume link lives under it (AC-28, AC-50). Absent: the loopback
//!   address the room binds.
//! - `DISCORD_CLIENT_ID`, `DISCORD_CLIENT_SECRET`, `DISCORD_GUILD_ID`,
//!   `DISCORD_ROLE_ID` — the Discord application and the guild and role that
//!   may host (SPEC §8, T-10). All four required and non-empty; the three ids
//!   are Discord snowflakes (digits). The OAuth redirect URI is not a
//!   variable: it is `POPQUIZ_PUBLIC_URL` + [`crate::discord::CALLBACK_PATH`].
//! - [`crate::admin::VAR`] — the pipeline channel's token (SPEC §8.3, T-25),
//!   required in **every** build: missing or empty, the room does not start.

use std::fmt;
use std::net::{Ipv4Addr, SocketAddr};

use crate::rooms::Urls;

/// SPEC §13's take-it-home page, where the released wall points (§5.5). The
/// same address the static fallback links to (PQ-12, D-24): the page lives on
/// the meetup's own host, whichever host serves the room.
pub const TAKE_IT_HOME: &str = "https://popquiz.rustnyc.org/last";

const DEFAULT_PORT: u16 = 3000;

/// The binary's configuration. Holds the admin token and the Discord client
/// secret, so it is deliberately not `Debug`.
pub struct Config {
    pub bind: SocketAddr,
    pub urls: Urls,
    pub(crate) admin_token: crate::admin::AdminToken,
    // T-10 ------------------------------------------------------------------
    pub(crate) discord: crate::discord::Settings,
    // end T-10 --------------------------------------------------------------
}

/// Why the room will not start. Each names the variable and never echoes a
/// credential.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// `PORT` is not a port number from 1 to 65535.
    Port(String),
    /// `POPQUIZ_PUBLIC_URL` is not `http(s)://host[:port]`.
    PublicUrl(String),
    /// The admin token is missing or empty (SPEC §8.3). Every build needs it:
    /// without it the pipeline could schedule nothing.
    MissingAdminToken,
    // T-10 ------------------------------------------------------------------
    /// A `DISCORD_*` variable is missing or empty. Named, never echoed.
    MissingDiscord(&'static str),
    /// A Discord id variable is not a snowflake (digits). Named, never echoed.
    DiscordId(&'static str),
    // end T-10 --------------------------------------------------------------
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Port(v) => write!(f, "PORT must be a port number from 1 to 65535, not {v:?}"),
            ConfigError::PublicUrl(v) => write!(
                f,
                "POPQUIZ_PUBLIC_URL must be http:// or https:// and a host, with no path, query or fragment, not {v:?}"
            ),
            ConfigError::MissingAdminToken => write!(
                f,
                "{} must be set and non-empty: it is the pipeline's admin token (fly secrets set, SPEC 8.3)",
                crate::admin::VAR
            ),
            // T-10
            ConfigError::MissingDiscord(var) => write!(f, "{var} must be set and non-empty (the Discord application, SPEC 8)"),
            ConfigError::DiscordId(var) => write!(f, "{var} must be a Discord id: digits only"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// The configuration, from `var(name)`. Pure: no environment, no clock.
    pub fn from_vars(var: impl Fn(&str) -> Option<String>) -> Result<Config, ConfigError> {
        let (bind, port) = match var("PORT") {
            None => (SocketAddr::from((Ipv4Addr::LOCALHOST, DEFAULT_PORT)), DEFAULT_PORT),
            Some(raw) => match raw.trim().parse::<u16>() {
                Ok(p) if p != 0 => (SocketAddr::from((Ipv4Addr::UNSPECIFIED, p)), p),
                _ => return Err(ConfigError::Port(raw)),
            },
        };
        let base = match var("POPQUIZ_PUBLIC_URL") {
            None => format!("http://127.0.0.1:{port}"),
            Some(raw) => public_base(&raw).ok_or(ConfigError::PublicUrl(raw))?,
        };
        let admin_token = crate::admin::AdminToken::from_var(var(crate::admin::VAR))?;
        // T-10
        let discord = discord_settings(&var, &base)?;
        Ok(Config {
            bind,
            urls: Urls {
                base,
                home: TAKE_IT_HOME.to_string(),
            },
            admin_token,
            discord,
        })
    }

    /// T-10: the OAuth redirect URI the Discord application must register.
    pub fn redirect_uri(&self) -> &str {
        &self.discord.redirect_uri
    }

    /// [`Config::from_vars`] over the process environment. A variable set to
    /// the empty string counts as unset.
    pub fn from_env() -> Result<Config, ConfigError> {
        Config::from_vars(|name| std::env::var(name).ok().filter(|v| !v.is_empty()))
    }
}

/// `scheme://host[:port]`, with one trailing `/` allowed and dropped. Anything
/// after the authority would end up in the middle of every join link.
fn public_base(raw: &str) -> Option<String> {
    let url = raw.trim();
    let url = url.strip_suffix('/').unwrap_or(url);
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let authority_ok = !rest.is_empty()
        && rest
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':' | '[' | ']'));
    authority_ok.then(|| url.to_string())
}

// T-10 ----------------------------------------------------------------------

/// The four Discord variables, in the order they are checked.
pub const DISCORD_VARS: [&str; 4] = ["DISCORD_CLIENT_ID", "DISCORD_CLIENT_SECRET", "DISCORD_GUILD_ID", "DISCORD_ROLE_ID"];

fn discord_settings(var: &impl Fn(&str) -> Option<String>, base: &str) -> Result<crate::discord::Settings, ConfigError> {
    let required = |name: &'static str| {
        var(name)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .ok_or(ConfigError::MissingDiscord(name))
    };
    let id = |name: &'static str| {
        let v = required(name)?;
        if v.bytes().all(|b| b.is_ascii_digit()) {
            Ok(v)
        } else {
            Err(ConfigError::DiscordId(name))
        }
    };
    let [client_id, client_secret, guild_id, role_id] = DISCORD_VARS;
    Ok(crate::discord::Settings {
        client_id: id(client_id)?,
        client_secret: crate::discord::Secret::new(required(client_secret)?),
        guild_id: id(guild_id)?,
        role_id: id(role_id)?,
        redirect_uri: format!("{base}{}", crate::discord::CALLBACK_PATH),
    })
}

// end T-10 ------------------------------------------------------------------
