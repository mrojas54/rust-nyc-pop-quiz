//! Clubs: several meetups on one room server (D-26, AC-103…105).
//!
//! One server, one Discord guild, one question bank, and a **host role per
//! club**. A club is `{slug, role ID, zone}`: the slug is what the host page
//! and the used ledger name it by, the role ID is what [`crate::discord`]
//! compares against `roles` (AC-65: an id, never a name), and the zone is where
//! the club's meetup date is read (a Pacific meetup at 9 pm is still that day).
//!
//! The default club is [`DEFAULT_CLUB`], `nyc`: what every link and request
//! that names no club means, so a link made before D-26 keeps working.

use std::fmt;

/// The club a request that names none belongs to.
pub const DEFAULT_CLUB: &str = "nyc";

/// A club's short name: `[a-z0-9-]`, 1 to 24 bytes. It travels in URLs, the
/// OAuth state and the used ledger, so anything else is refused before it
/// could reach a redirect (as [`crate::discord::is_question_id`] does for ids).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClubSlug(String);

impl ClubSlug {
    pub fn parse(raw: &str) -> Option<ClubSlug> {
        let ok = (1..=24).contains(&raw.len())
            && raw.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            && !raw.starts_with('-');
        ok.then(|| ClubSlug(raw.to_string()))
    }

    /// The default club's slug.
    pub fn default_club() -> ClubSlug {
        ClubSlug(DEFAULT_CLUB.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ClubSlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl serde::Serialize for ClubSlug {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

/// The zones a club may meet in: the four US zones that share the US daylight
/// rule (second Sunday of March to first Sunday of November). There is no
/// date crate here, so a zone is its standard offset and that one rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Zone {
    NewYork,
    Chicago,
    Denver,
    LosAngeles,
}

impl Zone {
    pub fn parse(name: &str) -> Option<Zone> {
        Some(match name {
            "America/New_York" => Zone::NewYork,
            "America/Chicago" => Zone::Chicago,
            "America/Denver" => Zone::Denver,
            "America/Los_Angeles" => Zone::LosAngeles,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Zone::NewYork => "America/New_York",
            Zone::Chicago => "America/Chicago",
            Zone::Denver => "America/Denver",
            Zone::LosAngeles => "America/Los_Angeles",
        }
    }

    /// Hours behind UTC in standard time.
    pub(crate) fn standard_hours_behind(self) -> i64 {
        match self {
            Zone::NewYork => 5,
            Zone::Chicago => 6,
            Zone::Denver => 7,
            Zone::LosAngeles => 8,
        }
    }
}

/// One club.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Club {
    pub slug: ClubSlug,
    /// The Discord role ID that may host this club's rooms (digits).
    pub role_id: String,
    pub zone: Zone,
}

/// Why `POPQUIZ_CLUBS` did not parse. Names the problem, never a role id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClubsError {
    /// An entry is not `slug=roleid:Zone`.
    Entry(String),
    /// A slug is not `[a-z0-9-]`, 1 to 24.
    Slug(String),
    /// A role id is not digits.
    RoleId(String),
    /// A zone is not one of the four supported.
    Zone(String),
    /// Two entries share a slug, or share a role (a role that hosted two clubs
    /// would let one club's organizers host the other's).
    Duplicate(String),
}

impl fmt::Display for ClubsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClubsError::Entry(e) => write!(f, "club entry {e:?} is not slug=roleid:Zone"),
            ClubsError::Slug(s) => write!(f, "club slug {s:?} must be 1 to 24 of a-z, 0-9, -"),
            ClubsError::RoleId(s) => write!(f, "the role id for club {s:?} must be digits only"),
            ClubsError::Zone(s) => write!(
                f,
                "the zone for club {s:?} must be America/New_York, America/Chicago, America/Denver or America/Los_Angeles"
            ),
            ClubsError::Duplicate(s) => write!(f, "club {s:?} repeats a slug or a role id: each club needs its own"),
        }
    }
}

impl std::error::Error for ClubsError {}

/// Every club this server runs. Never empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clubs(Vec<Club>);

impl Clubs {
    /// The default club alone: the server as it was before D-26.
    pub fn only_default(role_id: String) -> Clubs {
        Clubs(vec![Club {
            slug: ClubSlug::default_club(),
            role_id,
            zone: Zone::NewYork,
        }])
    }

    /// The default club's role (`DISCORD_ROLE_ID`) plus `extra`, the value of
    /// `POPQUIZ_CLUBS`: comma-separated `slug=roleid:Zone`, e.g.
    /// `la=123456789012345678:America/Los_Angeles`.
    pub fn parse(default_role: String, extra: Option<&str>) -> Result<Clubs, ClubsError> {
        let mut clubs = Clubs::only_default(default_role).0;
        for entry in extra.unwrap_or("").split(',').map(str::trim).filter(|e| !e.is_empty()) {
            let (slug, rest) = entry.split_once('=').ok_or_else(|| ClubsError::Entry(entry.to_string()))?;
            let (role, zone) = rest.split_once(':').ok_or_else(|| ClubsError::Entry(entry.to_string()))?;
            let slug = ClubSlug::parse(slug.trim()).ok_or_else(|| ClubsError::Slug(slug.trim().to_string()))?;
            let role = role.trim();
            if role.is_empty() || !role.bytes().all(|b| b.is_ascii_digit()) {
                return Err(ClubsError::RoleId(slug.to_string()));
            }
            let zone = Zone::parse(zone.trim()).ok_or_else(|| ClubsError::Zone(slug.to_string()))?;
            if clubs.iter().any(|c| c.slug == slug || c.role_id == role) {
                return Err(ClubsError::Duplicate(slug.to_string()));
            }
            clubs.push(Club {
                slug,
                role_id: role.to_string(),
                zone,
            });
        }
        Ok(Clubs(clubs))
    }

    pub fn get(&self, slug: &ClubSlug) -> Option<&Club> {
        self.0.iter().find(|c| &c.slug == slug)
    }

    pub fn all(&self) -> &[Club] {
        &self.0
    }

    /// The zone a club's meetup date is read in; the default club's for one
    /// this server does not know (a ledger line is never written for one).
    pub fn zone_of(&self, slug: &ClubSlug) -> Zone {
        self.get(slug).map_or(Zone::NewYork, |c| c.zone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_are_short_lowercase_names() {
        assert!(ClubSlug::parse("nyc").is_some());
        assert!(ClubSlug::parse("la-2").is_some());
        for bad in ["", "NYC", "-la", "la/", "la?x", "a b", &"a".repeat(25)] {
            assert!(ClubSlug::parse(bad).is_none(), "{bad:?}");
        }
    }

    #[test]
    fn the_default_role_alone_is_the_pre_d26_server() {
        let c = Clubs::parse("111".into(), None).unwrap();
        assert_eq!(c.all().len(), 1);
        assert_eq!(c.all()[0].slug.as_str(), "nyc");
        assert_eq!(c.all()[0].zone, Zone::NewYork);
    }

    #[test]
    fn a_second_club_has_its_own_role_and_zone() {
        let c = Clubs::parse("111".into(), Some("la=222:America/Los_Angeles")).unwrap();
        let la = c.get(&ClubSlug::parse("la").unwrap()).unwrap();
        assert_eq!((la.role_id.as_str(), la.zone), ("222", Zone::LosAngeles));
    }

    #[test]
    fn bad_entries_are_named_and_never_echo_a_role() {
        let bad = [
            ("la", ClubsError::Entry("la".into())),
            ("la=222", ClubsError::Entry("la=222".into())),
            ("LA=222:America/Los_Angeles", ClubsError::Slug("LA".into())),
            ("la=2x2:America/Los_Angeles", ClubsError::RoleId("la".into())),
            ("la=222:Mars/Olympus", ClubsError::Zone("la".into())),
            ("nyc=222:America/New_York", ClubsError::Duplicate("nyc".into())),
            ("la=111:America/Los_Angeles", ClubsError::Duplicate("la".into())),
        ];
        for (input, want) in bad {
            assert_eq!(Clubs::parse("111".into(), Some(input)).unwrap_err(), want, "{input}");
        }
        let shown = ClubsError::RoleId("la".into()).to_string();
        assert!(!shown.contains("222"));
    }
}
