#[cfg(feature = "fallback-deunicode")]
use deunicode::deunicode;

// include the python bindings module when building with the `python` feature
#[cfg(feature = "python")]
mod lib_py;

// If build.rs generated a table, include it. The file will define
// `static UNIDECODE_TABLE: phf::Map<u32, &'static str>`
#[allow(dead_code)]
// The build script writes `src/unidecode_table/mod.rs` and per-block files before compilation.
// Declare the module so we can call `unidecode_table::lookup(cp)` at runtime.
// Include the generated dispatcher module explicitly to avoid filesystem module
// discovery rules (we may have an old `src/unidecode_table.rs` marker file).
mod unidecode_table {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/unidecode_table/mod.rs"));
}

// Pure rust helper function for transliteration
pub fn unidecode_rust(input: &str) -> String {
    // If UNIDECODE_TABLE is available, use it per-codepoint to exactly match Python Unidecode.
    // We detect availability at compile time by checking for the symbol via cfg; since
    // conditional include above is cfg(any()) placeholder, we fallback dynamically.

    // Runtime: try to use the generated map via `UNIDECODE_TABLE` if present (linker will fail if not),
    // so as a robust approach, attempt per-codepoint lookup and fallback to deunicode.
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        let cp = ch as u32;
        if let Some(s) = unidecode_table::lookup(cp) {
            out.push_str(s);
            continue;
        }
        #[cfg(feature = "fallback-deunicode")]
        {
            let s = deunicode(&ch.to_string());
            if !s.is_empty() {
                out.push_str(&s);
            }
        }
        // Sans la feature fallback, on ignore (comportement identique à Python Unidecode qui retourne "" quand pas de translit)
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(unidecode_rust("déjà"), "deja");
    }
}
