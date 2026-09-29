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
//! - `HOST_DEV_TOKEN` — read **only** in a `dev-host-token` build, by
//!   [`crate::standin`]. Without the feature this module never names it.
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

/// The binary's configuration. Holds the admin token, and the stand-in's
/// credential in a `dev-host-token` build, so it is deliberately not `Debug`.
pub struct Config {
    pub bind: SocketAddr,
    pub urls: Urls,
    pub(crate) admin_token: crate::admin::AdminToken,
    #[cfg(feature = "dev-host-token")]
    pub(crate) host_token: crate::standin::DevHostToken,
}

/// Why the room will not start. Each names the variable and never echoes a
/// credential.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// `PORT` is not a port number from 1 to 65535.
    Port(String),
    /// `POPQUIZ_PUBLIC_URL` is not `http(s)://host[:port]`.
    PublicUrl(String),
    /// A `dev-host-token` build with `HOST_DEV_TOKEN` missing or empty. A
    /// stand-in that silently denied everyone would look like a broken host
    /// page, so it is a startup error instead. Only that build has it.
    #[cfg(feature = "dev-host-token")]
    MissingHostToken,
    /// The admin token is missing or empty (SPEC §8.3). Every build needs it:
    /// without it the pipeline could schedule nothing.
    MissingAdminToken,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Port(v) => write!(f, "PORT must be a port number from 1 to 65535, not {v:?}"),
            ConfigError::PublicUrl(v) => write!(
                f,
                "POPQUIZ_PUBLIC_URL must be http:// or https:// and a host, with no path, query or fragment, not {v:?}"
            ),
            #[cfg(feature = "dev-host-token")]
            ConfigError::MissingHostToken => write!(f, "this build has the dev-host-token feature, so {} must be set and non-empty", crate::standin::VAR),
            ConfigError::MissingAdminToken => write!(
                f,
                "{} must be set and non-empty: it is the pipeline's admin token (fly secrets set, SPEC 8.3)",
                crate::admin::VAR
            ),
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
        #[cfg(feature = "dev-host-token")]
        let host_token = crate::standin::DevHostToken::from_var(var(crate::standin::VAR))?;
        // Last, so a stand-in build reports its own missing token first.
        let admin_token = crate::admin::AdminToken::from_var(var(crate::admin::VAR))?;
        Ok(Config {
            bind,
            urls: Urls {
                base,
                home: TAKE_IT_HOME.to_string(),
            },
            admin_token,
            #[cfg(feature = "dev-host-token")]
            host_token,
        })
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
