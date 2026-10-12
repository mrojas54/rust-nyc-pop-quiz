//! The Discord backend's configuration: the application's settings and
//! secrets, where Discord is, the retry budget, and where log lines go.
//! Re-exported from [`super`], so each type keeps its `room::discord::…` path.

use std::time::Duration;

use subtle::ConstantTimeEq;

/// A secret string. No `Debug`, `Display` or `Clone`, so it cannot be
/// formatted or copied into anything by accident.
pub struct Secret(Box<str>);

impl Secret {
    pub fn new(value: String) -> Secret {
        Secret(value.into_boxed_str())
    }

    pub(super) fn expose(&self) -> &str {
        &self.0
    }

    pub(super) fn matches(&self, other: &str) -> bool {
        bool::from(self.0.as_bytes().ct_eq(other.as_bytes()))
    }
}

/// The Discord application and the guild it guards. Built by
/// [`crate::config`] from the four `DISCORD_*` variables and the public URL.
pub struct Settings {
    pub client_id: String,
    pub client_secret: Secret,
    pub guild_id: String,
    /// The clubs this server runs and each one's host role (D-26).
    pub clubs: crate::club::Clubs,
    /// `{POPQUIZ_PUBLIC_URL}{CALLBACK_PATH}`.
    pub redirect_uri: String,
}

/// Where Discord is. [`Endpoints::discord`] in the binary; the tests' mock
/// otherwise. Not configurable from the environment.
pub struct Endpoints {
    /// The API base, e.g. `https://discord.com/api/v10`.
    pub api: String,
    /// The consent screen, e.g. `https://discord.com/oauth2/authorize`.
    pub authorize: String,
}

impl Endpoints {
    pub fn discord() -> Endpoints {
        Endpoints {
            api: "https://discord.com/api/v10".into(),
            authorize: "https://discord.com/oauth2/authorize".into(),
        }
    }

    /// A Discord lookalike at `base` (`http://127.0.0.1:<port>`): the API under
    /// `/api/v10`, the consent screen at `/oauth2/authorize`.
    pub fn at(base: &str) -> Endpoints {
        let base = base.trim_end_matches('/');
        Endpoints {
            api: format!("{base}/api/v10"),
            authorize: format!("{base}/oauth2/authorize"),
        }
    }
}

/// How hard a check tries before it calls Discord unavailable.
#[derive(Clone, Copy, Debug)]
pub struct Retry {
    /// Tries per Discord call, the first included.
    pub attempts: u32,
    /// The wait before the second try; doubled for each one after.
    pub first_backoff: Duration,
    /// One try's bound, connection to last byte.
    pub per_attempt: Duration,
    /// The whole check's bound, every call and wait included.
    pub deadline: Duration,
}

impl Default for Retry {
    fn default() -> Retry {
        Retry {
            attempts: 3,
            first_backoff: Duration::from_millis(250),
            per_attempt: Duration::from_secs(3),
            deadline: Duration::from_secs(8),
        }
    }
}

/// Where the backend's log lines go. One line per Discord call: which call,
/// its status or that there was none, and the attempt. Never a token, a
/// header, a body, a code or a state.
pub trait Log: Send + Sync {
    fn line(&self, line: &str);
}

/// Standard error, which Fly collects.
pub struct Stderr;

impl Log for Stderr {
    fn line(&self, line: &str) {
        eprintln!("{line}");
    }
}
