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

/// Core transliteration (bit-for-bit equivalent to Python Unidecode for all mapped codepoints).
///
/// Current micro-optimisations:
/// - ASCII fast path: if the whole string is ASCII we return a direct clone.
/// - Heuristic pre-allocation (~2x input length) for mixed / non-ASCII text.
/// - Direct char iteration after an initial ASCII rejection (room for SIMD scan later).
pub fn unidecode(input: &str) -> String {
    // Fast path: pure ASCII -> identical output.
    if input.is_ascii() {
        return input.to_string();
    }

    // Pass 1: estimate resulting length & collect ASCII runs cheaply.
    // We walk UTF-8 decoding minimally using .chars() (still efficient) but track expansion sizes.
    // For now we approximate expansion length as len(mapping) else 0 (or 1 for ASCII). This avoids
    // repeated reallocations for CJK multi-letter expansions.
    let mut estimated = 0usize;
    for ch in input.chars() {
        let cp = ch as u32;
        if cp < 0x80 {
            estimated += 1; // ASCII maps to itself.
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
    out
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
}
