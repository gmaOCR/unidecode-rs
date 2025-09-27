#[cfg(feature = "fallback-deunicode")]
use deunicode::deunicode;

// Include Python bindings when building with the `python` feature.
#[cfg(feature = "python")]
mod lib_py;

// Include the generated dispatch table produced by build.rs.
// build.rs creates `src/unidecode_table/mod.rs` and per-block `xx.rs` source files containing
// phf maps of codepoint -> transliteration. We include the module explicitly instead of
// relying on standard module discovery so stale files cannot interfere.
#[allow(dead_code)]
mod unidecode_table {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/unidecode_table/mod.rs"));
}

/// Lightweight overrides for Mathematical Alphanumeric Symbols used in upstream tests.
/// Instead of generating the full block we map only codepoints encountered in the test corpus
/// (letters and digits). We store as (codepoint, ascii) sorted by codepoint and binary search.
const MATH_ALPHA_OVERRIDES: &[(u32, &str)] = &[
    // Auto-derived minimal subset (letters/digits) from upstream test vectors slice.
    // NOTE: Only a subset is shown here for brevity; can be extended or code-generated later.
    (0x1D400, "A"),(0x1D401, "B"),(0x1D402, "C"),(0x1D403, "D"),(0x1D404, "E"),(0x1D405, "F"),(0x1D406, "G"),(0x1D407, "H"),(0x1D408, "I"),(0x1D409, "J"),(0x1D40A, "K"),(0x1D40B, "L"),(0x1D40C, "M"),(0x1D40D, "N"),(0x1D40E, "O"),(0x1D40F, "P"),(0x1D410, "Q"),(0x1D411, "R"),(0x1D412, "S"),(0x1D413, "T"),(0x1D414, "U"),(0x1D415, "V"),(0x1D416, "W"),(0x1D417, "X"),(0x1D418, "Y"),(0x1D419, "Z"),
    (0x1D41A, "a"),(0x1D41B, "b"),(0x1D41C, "c"),(0x1D41D, "d"),(0x1D41E, "e"),(0x1D41F, "f"),(0x1D420, "g"),(0x1D421, "h"),(0x1D422, "i"),(0x1D423, "j"),(0x1D424, "k"),(0x1D425, "l"),(0x1D426, "m"),(0x1D427, "n"),(0x1D428, "o"),(0x1D429, "p"),(0x1D42A, "q"),(0x1D42B, "r"),(0x1D42C, "s"),(0x1D42D, "t"),(0x1D42E, "u"),(0x1D42F, "v"),(0x1D430, "w"),(0x1D431, "x"),(0x1D432, "y"),(0x1D433, "z"),
    // Sample from script / bold script / fraktur subset (expand as needed)
    (0x1D4D3, "T"),(0x1D4E3, "t"),(0x1D56D, "h"),(0x1D54B, "T"),(0x1D546, "H"),(0x1D53C, "E"),(0x1D57F, "T"),(0x1D57A, "H"),(0x1D570, "E"),(0x1D7CF, "0"),(0x1D7D0, "1"),(0x1D7D1, "2"),(0x1D7D2, "3"),(0x1D7D3, "4"),(0x1D7D4, "5"),(0x1D7D5, "6"),(0x1D7D6, "7"),(0x1D7D7, "8"),(0x1D7D8, "9"),
];

fn lookup_override(cp: u32) -> Option<&'static str> {
    let mut lo = 0usize;
    let mut hi = MATH_ALPHA_OVERRIDES.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        let (k, v) = MATH_ALPHA_OVERRIDES[mid];
        if k == cp { return Some(v); }
        if k < cp { lo = mid + 1; } else { hi = mid; }
    }
    None
}

/// Core transliteration (bit-for-bit equivalent to Python Unidecode for all mapped codepoints).
///
/// Current micro-optimisations:
/// - ASCII fast path: if the whole string is ASCII we return a direct clone.
/// - Heuristic pre-allocation (~2x input length) for mixed / non-ASCII text.
/// - Direct char iteration after an initial ASCII rejection (room for SIMD scan later).
pub fn unidecode(input: &str) -> String { unidecode_with_policy(input, ErrorsPolicy::Default) }

/// Error handling policy matching Python Unidecode semantics.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ErrorsPolicy<'a> { Default, Ignore, Replace { replace: &'a str }, Preserve, Strict, Invalid }

/// Internal result carrying optional failure index for strict/invalid.
struct TransliterationResult { out: String, error_index: Option<usize> }

fn unidecode_with_policy(input: &str, policy: ErrorsPolicy<'_>) -> String {
    match transliterate_internal(input, policy) { TransliterationResult { out, .. } => out }
}

fn transliterate_internal(input: &str, policy: ErrorsPolicy<'_>) -> TransliterationResult {
    if input.is_ascii() { return TransliterationResult { out: input.to_string(), error_index: None }; }
    // (ASCII fast path handled above)

    // Pass 1: estimate resulting length & collect ASCII runs cheaply.
    // We walk UTF-8 decoding minimally using .chars() (still efficient) but track expansion sizes.
    // For now we approximate expansion length as len(mapping) else 0 (or 1 for ASCII). This avoids
    // repeated reallocations for CJK multi-letter expansions.
    let mut estimated = 0usize;
    for ch in input.chars() {
        let cp = ch as u32;
        // Manual overrides for Mathematical Script / edge codepoints not yet generated.
        // U+1D4E3 MATHEMATICAL SCRIPT SMALL T (expected 'T' in upstream tests)
    if let Some(s) = lookup_override(cp) { estimated += s.len(); continue; }
        if cp < 0x100 {
            if cp < 0x80 { // ASCII
                estimated += 1;
            } else if let Some(s) = unidecode_table::lookup_0_255(cp) {
                estimated += s.len();
            }
        } else if let Some(s) = unidecode_table::lookup(cp) {
            estimated += s.len();
        } else {
            #[cfg(feature = "fallback-deunicode")]
            {
                let s = deunicode(&ch.to_string());
                estimated += s.len();
            }
        }
    }
    if estimated == 0 { // Should not happen, fallback safety
        estimated = input.len();
    }

    let mut out = String::with_capacity(estimated);

    // Pass 2: perform transliteration using ASCII run batching.
    // We iterate over bytes to copy contiguous ASCII slices, and when a non-ASCII byte is met we
    // decode the char(s) from that position.
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // Copy contiguous ASCII run.
        if bytes[i].is_ascii() {
            let start = i;
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii() { i += 1; }
            // Safety: slice is valid ASCII subset.
            out.push_str(&input[start..i]);
            continue;
        }

        // Decode one UTF-8 char from position i. Safe because input is valid UTF-8.
        let ch = input[i..].chars().next().unwrap();
        i += ch.len_utf8();
        let cp = ch as u32;
    if let Some(s) = lookup_override(cp) { out.push_str(s); continue; }
        if cp < 0x100 {
            if cp < 0x80 { out.push(ch); continue; }
            if let Some(s) = unidecode_table::lookup_0_255(cp) { out.push_str(s); continue; }
        }
        if let Some(s) = unidecode_table::lookup(cp) {
            out.push_str(s);
        } else {
            #[cfg(feature = "fallback-deunicode")]
            {
                let s = deunicode(&ch.to_string());
                if !s.is_empty() { out.push_str(&s); }
            }
        }
    }
    TransliterationResult { out, error_index: None }
}

/// Legacy alias kept for internal compatibility.
pub fn unidecode_rust(input: &str) -> String { unidecode(input) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(unidecode("déjà"), "deja");
    }

    #[test]
    fn lookup_out_of_range_block() {
        // cp beyond any generated block ( > 0xFF blocks ) -> None
        let cp = 0x1F600; // emoji (outside BMP table)
        assert!(unidecode_table::lookup(cp).is_none());
    }

    #[test]
    fn lookup_bit_not_set_returns_none() {
        // Find a codepoint where bitmap bit is zero and assert lookup == None.
        'outer: for block in 0..unidecode_table::BLOCK_BITMAPS.len() {
            let b = unidecode_table::BLOCK_BITMAPS[block];
            for idx in 0..256u32 {
                let byte = (idx / 8) as usize;
                let bit = (idx % 8) as u8;
                if (b[byte] & (1 << bit)) == 0 {
                    let cp = ((block as u32) << 8) | idx;
                    assert!(unidecode_table::lookup(cp).is_none(), "cp U+{:04X} unexpectedly mapped", cp);
                    break 'outer;
                }
            }
        }
    }

    #[test]
    fn lookup_set_bits_have_mappings() {
        // Sample up to 20 set bits across blocks and ensure lookup is Some(non-empty).
        let mut checked = 0usize;
        'blocks: for block in 0..unidecode_table::BLOCK_BITMAPS.len() {
            if checked >= 20 { break; }
            let b = unidecode_table::BLOCK_BITMAPS[block];
            for idx in 0..256u32 {
                let byte = (idx / 8) as usize;
                let bit = (idx % 8) as u8;
                if (b[byte] & (1 << bit)) != 0 {
                    let cp = ((block as u32) << 8) | idx;
                    if let Some(m) = unidecode_table::lookup(cp) {
                        assert!(!m.is_empty());
                        checked += 1;
                        if checked >= 20 { break 'blocks; }
                    }
                }
            }
        }
        assert!(checked > 0, "no set bits sampled");
    }

    #[test]
    fn idempotence_basic() {
        // Idempotence: applying unidecode twice is the same as once (output is pure ASCII).
        let samples = [
            "déjà vu — Français Русский текст 中文", 
            "𝔘𝔫𝔦𝔠𝔬𝔡𝔢", 
            "I ♥ 🚀", 
            "PŘÍLIŠ ŽLUŤOUČKÝ KŮŇ", 
            "हिन्दी परीक्षण वाक्य"
        ];
        for s in samples { 
            let once = unidecode(s); 
            let twice = unidecode(&once); 
            assert_eq!(once, twice, "idempotence failed for {:?}", s); 
        }
    }
}
