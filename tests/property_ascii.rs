//! Property-based tests ensuring transliteration output constraints.
use proptest::prelude::*;
use unidecode_rs::unidecode;

// Helper invoking internal policy function via public API variants where possible.
fn is_ascii(s: &str) -> bool {
    s.bytes().all(|b| b.is_ascii())
}

proptest! {
    #[test]
    fn output_is_ascii_default(ref s in "(?s).{0,256}") { // arbitrary up to 256 bytes
        let out = unidecode(s);
        prop_assert!(is_ascii(&out));
    }
}

proptest! {
    #[test]
    fn idempotent_default(ref s in "(?s).{0,256}") {
        let out1 = unidecode(s);
        let out2 = unidecode(&out1);
        prop_assert_eq!(out1, out2);
    }
}

proptest! {
    #[test]
    fn no_panics_random_unicode(ref s in proptest::collection::vec(any::<char>(), 0..128)) {
        let input: String = s.iter().collect();
        let out = unidecode(&input);
        // Guarantee ASCII; check length upper bound heuristic (not strict but sanity: <= 4x input bytes)
        prop_assert!(is_ascii(&out));
        prop_assert!(out.len() <= input.len() * 4 + 8);
    }
}
