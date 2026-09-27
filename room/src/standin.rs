//! SPEC §8.2's stand-in (D-19, T-09): *Create a room* before Discord.
//!
//! One shared secret, `HOST_DEV_TOKEN`, set as a Fly secret and carried by the
//! host page's URL fragment (`/host?question=q3#<token>`), which the page sends
//! as the bearer on *Create a room*. [`DevHostToken`] is a backend of the same
//! auth function every other create check goes through ([`HostAuth`], G-9):
//!
//! - it accepts exactly that bearer for *Create a room* and *Run it again* —
//!   the only two calls that ask `authorize_create` — compared in constant time;
//! - it grants nothing else: host commands keep the trait's default, the room's
//!   own host session;
//! - it logs nothing, and the type has no `Debug`, `Display` or `Clone`, so
//!   the token cannot be formatted by accident.
//!
//! The HC-0 question is seeded here too: q3, the one migrated verified record,
//! embedded at compile time. Nothing is read from disk at run time, and the
//! record is the verifier's, never edited by hand.
//!
//! This whole module exists only with the `dev-host-token` feature; so does
//! every line that names it (`tests/standin.rs` holds both to that). T-10
//! deletes the feature, this file and those lines.

#![cfg(feature = "dev-host-token")]

use subtle::ConstantTimeEq;

use crate::answers::{self, Scheduled};
use crate::auth::{Denied, HostAuth, OrganizerId};
use crate::config::ConfigError;

/// The variable the secret is read from — the one place in `src/` it is named.
pub const VAR: &str = "HOST_DEV_TOKEN";

/// The one organizer the stand-in knows. *Run it again* requires the same
/// organizer as the room it follows (AC-68), and with one token there is one.
pub const ORGANIZER: &str = "dev-host-token";

/// The shared secret, held as bytes. No `Debug`, `Display` or `Clone`.
pub struct DevHostToken {
    token: Box<[u8]>,
}

impl DevHostToken {
    /// The stand-in from the variable's value. Missing or empty is an error:
    /// a stand-in that accepted nothing would look like a broken host page.
    pub fn from_var(value: Option<String>) -> Result<DevHostToken, ConfigError> {
        match value {
            Some(v) if !v.trim().is_empty() => Ok(DevHostToken {
                token: v.into_bytes().into_boxed_slice(),
            }),
            _ => Err(ConfigError::MissingHostToken),
        }
    }
}

impl HostAuth for DevHostToken {
    fn authorize_create(&self, bearer: Option<&str>) -> Result<OrganizerId, Denied> {
        // `ct_eq` on slices of different lengths is `false` without looking at
        // the bytes; the length of a 64-hex-character token is not the secret.
        match bearer {
            Some(b) if bool::from(self.token.ct_eq(b.as_bytes())) => Ok(OrganizerId(ORGANIZER.into())),
            _ => Err(Denied),
        }
    }
}

/// The HC-0 question: `bank/questions/q3.json`, as the verifier wrote it.
pub fn hc0_question() -> Scheduled {
    answers::load(include_str!("../../bank/questions/q3.json"))
        .unwrap_or_else(|e| panic!("bank/questions/q3.json does not load: {e}"))
}
