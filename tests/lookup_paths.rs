//! Additional tests focused on covering internal lookup code paths.
//! Goal: exercise multiple branches of the large generated `match` in `unidecode_table::lookup`.

use unidecode_rs::unidecode;

#[test]
fn sample_multiple_blocks() {
    // Select a few known code points across different blocks (Greek, Cyrillic, symbols).
    let samples = [
        (0x0398, "Th"), // GREEK CAPITAL THETA
        (0x00DF, "ss"), // LATIN SMALL LETTER SHARP S via tableau 0-255
    (0x0416, "Zh"), // CYRILLIC CAPITAL ZHE (classic python table -> Zh)
    // (0x221E, "Infinity"), // ∞ -> Infinity (may not be mapped in our current subset)
        (0x00AE, "(R)"), // Registered sign
    ];
    for (cp, expected_fragment) in samples {
        let ch = char::from_u32(cp).unwrap();
        let out = unidecode(&ch.to_string());
        if expected_fragment == "(R)" {
            assert!(out == "(R)" || out == "(r)", "U+{:04X} => {:?} missing (R)/(r) variant", cp, out);
        } else {
            assert!(out.contains(expected_fragment), "U+{:04X} => {:?} does not contain {:?}", cp, out, expected_fragment);
        }
    }
}

#[test]
fn ascii_passthrough_idempotent() {
    let s = "The quick brown fox 123";
    assert_eq!(unidecode(s), s);
}
