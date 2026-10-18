//! Whether a question is affirmed for the text the room was given (G-12,
//! AC-72, GAP-24, ADR-0005 item 9).
//!
//! **The rule is the pipeline's.** `pipeline/src/popquiz/bank.py`'s
//! `affirmation_digest` and `affirmation_state` are the reference, and this is
//! their twin, as `correct_index` and the receipt are (one rule, never two). Both
//! are held to the cases in `bank/fixtures/affirmation/`, by
//! `pipeline/tests/test_affirmation_twin.py` and by `tests/twins.rs`. If the
//! digest's inputs or its canonical form change, both sides change.
//!
//! The digest is SHA-256 over compact JSON with sorted keys of the record's
//! `source`, `options`, `hint`, `explains` and `trace`, built as `bank._dump`
//! builds them: absent and `null` are one thing, `pivot: false` is left out,
//! and every step has its `values`. [`crate::answers::Record`] builds that
//! payload; this module hashes it and says what the review makes of it.
//!
//! **Not security.** An affirmation is an organizer's sign-off, and the digest
//! binds it to the text it was given to. It does not prove who wrote it: anyone
//! who can edit the bank or holds the admin token can write one (ADR-0005,
//! failure mode 3).

use serde::Deserialize;

/// The bank's `review` block, every field the schema has. The room reads only
/// the affirmation; the rest is parsed so an unknown field is refused, as
/// everywhere else in the record.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Review {
    #[allow(dead_code)]
    status: Option<serde_json::Value>,
    #[allow(dead_code)]
    reason: Option<serde_json::Value>,
    #[allow(dead_code)]
    difficulty_judged: Option<serde_json::Value>,
    affirmed_by: Option<String>,
    affirmed_at: Option<String>,
    affirmed_digest: Option<String>,
    #[allow(dead_code)]
    quotes_checked: Option<Vec<String>>,
    #[allow(dead_code)]
    near_duplicate_of: Option<serde_json::Value>,
}

impl Review {
    /// The twin of `bank.Review.affirmed`: who *and* when, both non-empty.
    fn affirmed(&self) -> bool {
        self.affirmed_by.as_deref().is_some_and(|s| !s.is_empty())
            && self.affirmed_at.as_deref().is_some_and(|s| !s.is_empty())
    }

    /// The twin of `bank.validate_question`'s review checks: a digest must be
    /// `sha256:<64 hex>`, and must not stand without who and when. The `Err` is
    /// the reason, without the id.
    pub(crate) fn check_shape(&self) -> Result<(), &'static str> {
        let Some(digest) = &self.affirmed_digest else {
            return Ok(());
        };
        if !well_formed(digest) {
            return Err("review.affirmed_digest is not sha256:<64 hex>");
        }
        if !self.affirmed() {
            return Err("review.affirmed_digest is recorded without affirmed_by and affirmed_at");
        }
        Ok(())
    }
}

fn well_formed(digest: &str) -> bool {
    digest.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

/// Where a question's affirmation stands: `bank.affirmation_state`'s four
/// answers. Only [`Affirmation::Bound`] may run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Affirmation {
    /// Affirmed, and the text still matches the digest recorded then. `by` is
    /// the organizer who affirmed it, which the used record carries.
    Bound { by: String },
    /// Affirmed, and the text changed afterwards.
    Stale,
    /// Affirmed with no digest: who and when, but not what (the q3 ruling on
    /// PQ-23: it counts nowhere).
    Unbound,
    /// Nobody is recorded as having affirmed it.
    None,
}

impl Affirmation {
    /// The twin of `bank.affirmation_state`, given the digest of the record's
    /// text in the bank's order.
    pub(crate) fn of(review: Option<&Review>, digest: &str) -> Affirmation {
        let Some(review) = review.filter(|r| r.affirmed()) else {
            return Affirmation::None;
        };
        match &review.affirmed_digest {
            None => Affirmation::Unbound,
            Some(d) if d == digest => Affirmation::Bound {
                by: review.affirmed_by.clone().unwrap_or_default(),
            },
            Some(_) => Affirmation::Stale,
        }
    }

    /// `bank.AffirmationState`'s name for this state.
    pub fn state(&self) -> &'static str {
        match self {
            Affirmation::Bound { .. } => "bound",
            Affirmation::Stale => "stale",
            Affirmation::Unbound => "unbound",
            Affirmation::None => "none",
        }
    }

    /// The organizer who affirmed it, for a bound affirmation only.
    pub fn affirmed_by(&self) -> Option<&str> {
        match self {
            Affirmation::Bound { by } => Some(by),
            _ => None,
        }
    }

    /// Why the room will not run question `id`, or `None` when it may. For the
    /// organizer's pipeline (the admin route's `reason`) and the host's refusal;
    /// it quotes no question text.
    pub fn refusal(&self, id: &str) -> Option<String> {
        let why = match self {
            Affirmation::Bound { .. } => return None,
            Affirmation::Stale => "its text changed after it was affirmed; affirm it again at review.",
            Affirmation::Unbound => {
                "its affirmation records who and when but not what was affirmed; affirm it again at review."
            }
            Affirmation::None => "is not affirmed; affirm it at review, then schedule it again.",
        };
        Some(format!("{id}: {why}"))
    }
}

/// `sha256:<hex>` of `payload` as compact JSON with sorted keys: the twin of
/// `json.dumps(payload, sort_keys=True, ensure_ascii=False, separators=(",", ":"))`.
///
/// serde_json's maps are `BTreeMap`s here (no `preserve_order`), so keys come
/// out sorted, and its escapes are Python's: `\"`, `\\`, `\b \f \n \r \t`,
/// `\u00xx` in lower case for the other control characters, and everything
/// else as raw UTF-8. `tests/twins.rs` pins both against the shared fixtures.
///
/// The keys are also sorted here, by rebuilding each map in key order, so the
/// digest does not hang on that feature staying off.
pub(crate) fn digest_of(payload: &serde_json::Value) -> String {
    let text = serde_json::to_string(&sorted(payload)).expect("a JSON value serializes");
    let hash = ring::digest::digest(&ring::digest::SHA256, text.as_bytes());
    let hex: String = hash.as_ref().iter().map(|b| format!("{b:02x}")).collect();
    format!("sha256:{hex}")
}

fn sorted(value: &serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            Value::Object(
                keys.into_iter()
                    .map(|k| (k.clone(), sorted(&map[k])))
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.iter().map(sorted).collect()),
        other => other.clone(),
    }
}
