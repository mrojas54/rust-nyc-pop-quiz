//! The auth seam (SPEC.md G-9, §8).
//!
//! One trait answers the only two questions the room asks: *may this bearer
//! create a room?* and *may this bearer act as host of this room?* The first
//! is the credential path — the M1 stand-in (§8.2, T-09) and then Discord's
//! role-ID check (T-10) implement it. The second, by default, is the room's own
//! host session, which never rotates (AC-50) and is compared in constant time.
//!
//! A default build ships no implementation that accepts anything: [`DenyAll`]
//! is what `router()` and the binary use, so it can create no room. The one
//! that accepts a credential before T-10 is SPEC §8.2's stand-in,
//! `crate::standin`, which exists only with the `dev-host-token` feature
//! (T-09). The test implementation lives in `tests/common`.

use crate::rooms::Room;

/// Who created a room. Only that organizer controls it (AC-68).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrganizerId(pub String);

/// A refusal. It carries nothing, so it can say nothing about why (AC-70).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Denied;

pub trait HostAuth: Send + Sync + 'static {
    /// *Create a room* (and *Run it again*, which creates one).
    fn authorize_create(&self, bearer: Option<&str>) -> Result<OrganizerId, Denied>;

    /// Every other host command, and the host's view of the room.
    fn authorize_host(&self, room: &Room, bearer: Option<&str>) -> Result<(), Denied> {
        match bearer {
            Some(b) if room.host_matches(b) => Ok(()),
            _ => Err(Denied),
        }
    }
}

/// Authorizes nothing. What the room runs with until T-09 or T-10 plugs in a
/// real check.
pub struct DenyAll;

impl HostAuth for DenyAll {
    fn authorize_create(&self, _bearer: Option<&str>) -> Result<OrganizerId, Denied> {
        Err(Denied)
    }
}
