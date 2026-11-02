//! Shared between the two T-03 spike bins. Not a binary — `autobins = false`
//! in Cargo.toml is what keeps cargo from treating it as one.
//!
//! Everything here is computed on **both** sides of the wire and compared, so a
//! second copy that drifted would turn the AC-52 reconciliation into a check
//! that always passes. One definition, included by both bins.
//!
//! Each bin uses a subset, and `#[path]` inclusion means the unused half looks
//! dead to whichever one is being compiled. That is what the allow is for; it is
//! not covering up anything unreachable.
#![allow(dead_code)]

/// FNV-1a over the session id. Not a security hash and not meant to be one —
/// it only has to be the same function in both processes.
pub fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// SplitMix64. Used for the answer fingerprint and for the burst's per-client
/// offsets, so a run is reproducible from `--seed` alone.
pub fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// One session's contribution to the room's answer fingerprint.
///
/// The fingerprint is the XOR of this over every session. XOR is its own
/// inverse, so an upsert is `fp ^= old; fp ^= new` and the server never has to
/// walk the map to keep it current.
///
/// Why it exists: `applied_seq_sum` alone is a pooled sum, and two sessions
/// swapping answers can leave a sum unchanged. Binding the session id into each
/// term means a swap changes the fingerprint. The server still exposes no
/// per-session answer — only this one number — so the spike stays structurally
/// honest to AC-56 even though it is throwaway code.
pub fn answer_contribution(session: &str, letter: u8, seq: u32) -> u64 {
    splitmix64(fnv1a(session) ^ ((letter as u64) << 32) ^ (seq as u64))
}

/// `A`..`E` as 0..4. Returns `None` for anything else, which the server treats
/// as a malformed frame rather than guessing.
pub fn letter_index(letter: char) -> Option<u8> {
    match letter {
        'A'..='E' => Some(letter as u8 - b'A'),
        _ => None,
    }
}

/// 0..4 back to `A`..`E`.
pub fn index_letter(i: u8) -> char {
    (b'A' + i) as char
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_swap_between_two_sessions_changes_the_fingerprint() {
        // The exact case `applied_seq_sum` cannot see: two sessions exchange
        // answers, so every aggregate (per-letter totals, the seq sum, the
        // count) is unchanged.
        let straight = answer_contribution("s-0001", 0, 7) ^ answer_contribution("s-0002", 3, 7);
        let swapped = answer_contribution("s-0001", 3, 7) ^ answer_contribution("s-0002", 0, 7);
        assert_ne!(straight, swapped);
    }

    #[test]
    fn xor_undoes_itself_so_an_upsert_needs_no_rescan() {
        let mut fp = 0u64;
        fp ^= answer_contribution("s-0001", 0, 1);
        let old = answer_contribution("s-0001", 0, 1);
        fp ^= old; // retract
        fp ^= answer_contribution("s-0001", 4, 2); // apply the new one
        assert_eq!(fp, answer_contribution("s-0001", 4, 2));
    }

    #[test]
    fn letters_round_trip_and_reject_what_is_not_a_letter() {
        for i in 0..5u8 {
            assert_eq!(letter_index(index_letter(i)), Some(i));
        }
        assert_eq!(letter_index('F'), None);
        assert_eq!(letter_index('a'), None);
    }
}
