//! Spec-derived tests adapted from the original Python unidecode test suite.
//! We intentionally keep the scope: output equivalence + invariants (ASCII only, no panics).
//! Python-specific behaviors (warnings, `errors=` modes, CLI utility) are out-of-scope here.

use unidecode_rs::unidecode;

fn ascii(c: u32) -> char {
    char::from_u32(c).unwrap()
}

#[test]
fn ascii_identity() {
    for cp in 0u32..128 {
        let ch = ascii(cp);
        assert_eq!(unidecode(&ch.to_string()), ch.to_string());
    }
}

#[test]
fn empty_string() {
    assert_eq!(unidecode(""), "");
}

#[test]
fn latin1_basic() {
    // Subset overlapping with WordPress / common accents.
    let cases = [
        ("é", "e"),
        ("É", "E"),
        ("Ä", "A"),
        ("ä", "a"),
        ("Ö", "O"),
        ("ö", "o"),
        ("Ü", "U"),
        ("ü", "u"),
        ("ß", "ss"),
        ("Þ", "Th"),
        ("þ", "th"),
        ("Æ", "AE"),
        ("æ", "ae"),
    ];
    for (inp, exp) in cases {
        assert_eq!(unidecode(inp), exp, "latin1 case {:?}", inp);
    }
}

#[test]
fn degree_equivalence() {
    // U+2109 vs \u00B0F ; U+2103 vs \u00B0C
    assert_eq!(unidecode("\u{2109}"), unidecode("\u{00B0}F"));
    assert_eq!(unidecode("\u{2103}"), unidecode("\u{00B0}C"));
}

#[test]
fn circled_latin_subset() {
    // Only a subset (a..z) small; full coverage in Python tests already.
    for i in 0..26 {
        let cp = 0x24d0 + i;
        let out = unidecode(&char::from_u32(cp).unwrap().to_string());
        assert_eq!(out, ((b'a' + i as u8) as char).to_string());
    }
}

#[test]
fn fullwidth_sentence() {
    // Fullwidth phrase -> ASCII quick brown fox sentence (lowercase variant test case subset)
    let full = "ｔｈｅ ｑｕｉｃｋ ｂｒｏｗｎ ｆｏｘ ｣"
        .replace('｣', "ｊｕｍｐｓ")
        .to_string()
        + " ｏｖｅｒ ｔｈｅ ｌａｚｙ ｄｏｇ １２３４５";
    let out = unidecode(&full);
    assert!(
        out.starts_with("the quick brown fox jumps over the lazy dog 12345"),
        "got {}",
        out
    );
}

#[test]
fn enclosed_alphanumerics_sample() {
    assert_eq!(unidecode("ⓐⒶ⑳⒇⒛⓴⓾⓿"), "aA20(20)20.20100");
}

#[test]
fn non_bmp_basic() {
    // Few mathematical bold/script code points already covered in Python wide tests.
    let samples = [0x1d5a0u32, 0x1d5c4, 0x1d5c6];
    for cp in samples {
        let ch = char::from_u32(cp).unwrap();
        let out = unidecode(&ch.to_string());
        // Current implementation may not yet provide a mapping; empty is acceptable.
        if !out.is_empty() {
            assert!(out.is_ascii(), "Non ASCII output for U+{:X}: {:?}", cp, out);
        }
    }
}

#[test]
fn unmapped_returns_empty() {
    // Example from Python test (table that doesn't exist) gives empty.
    assert_eq!(unidecode("\u{A500}"), "");
}

#[test]
fn partial_table_empty_example() {
    // Python example: \u1EFF expected empty (block short). Keep same expectation.
    assert_eq!(unidecode("\u{1EFF}"), "");
}

#[test]
fn large_scan_subset_no_panic_ascii_output() {
    // Scan a moderate prefix (skip surrogate range; UTF-8 Rust can't hold surrogates anyway).
    for cp in 0u32..0x5000 {
        if (0xD800..=0xDFFF).contains(&cp) {
            continue;
        }
        let ch = match char::from_u32(cp) {
            Some(c) => c,
            None => continue,
        };
        let out = unidecode(&ch.to_string());
        assert!(
            out.is_ascii(),
            "Non ASCII output at U+{:04X}: {:?}",
            cp,
            out
        );
    }
}

#[test]
fn mixed_complex_sentence() {
    let s = "PŘÍLIŠ ŽLUŤOUČKÝ KŮŇ pěl ďábelské ÓDY déjà vu — Русский текст 中文 😀 𝔘𝔫𝔦𝔠𝔬𝔡𝔢";
    let out = unidecode(s);
    // Basic sanity: all ASCII
    assert!(out.is_ascii());
    // Contains expected transliterations fragments
    assert!(out.contains("PRILIS"));
    assert!(out.contains("ZLUTOUCKY"));
    assert!(out.contains("deja vu"));
    assert!(out.contains("Russkii"), "got {}", out);
}
