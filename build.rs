use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Build script: queries Python Unidecode block-by-block (256 code points per block)
    // and generates `xx.rs` files (hex block prefix) under src/unidecode_table/ plus a
    // dispatching mod.rs that exposes `lookup(cp)`.

    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out_dir = crate_root.join("src").join("unidecode_table");

    if !out_dir.exists() {
        fs::create_dir_all(&out_dir).expect("failed to create src/unidecode_table");
    }

    let python = env::var("PYTHON").unwrap_or_else(|_| "python3".to_string());

    // Collect direct Latin-1 table (0x00-0xFF). We only store non-ASCII (>=0x80) mappings.
    let mut latin1: Vec<Option<String>> = vec![None; 256];

    for block in 0u32..0x110u32 {
        let start = block << 8;
        let end = ((block + 1) << 8) - 1;

        let py_code = format!(r#"import json,sys
from unidecode import unidecode as _u
out={{}}
for cp in range({start},{end}+1):
    ch=chr(cp)
    s=_u(ch)
    if s:
        out[cp]=s
sys.stdout.reconfigure(encoding='utf-8')
print(json.dumps(out, ensure_ascii=False))"#, start = start, end = end);

        let output = Command::new(&python)
            .arg("-c")
            .arg(&py_code)
            .output()
            .expect("failed to run python to extract Unidecode block");

        let stdout = if output.status.success() {
            output.stdout
        } else {
            // Try to install Unidecode on demand and retry once.
            eprintln!("python extraction failed for block {:#x}; attempting pip install Unidecode", block);
            let install = Command::new(&python)
                .arg("-m")
                .arg("pip")
                .arg("install")
                .arg("Unidecode")
                .output()
                .expect("failed to run pip install Unidecode");
            if !install.status.success() {
                let stderr = String::from_utf8_lossy(&install.stderr);
                panic!("pip install failed: {}", stderr);
            }
            let output2 = Command::new(&python)
                .arg("-c")
                .arg(&py_code)
                .output()
                .expect("failed to run python to extract Unidecode block (retry)");
            if !output2.status.success() {
                let stderr = String::from_utf8_lossy(&output2.stderr);
                panic!("python extraction retry failed: {}", stderr);
            }
            output2.stdout
        };

        let json_text = String::from_utf8(stdout).expect("python returned non-utf8 output");
        write_block(&out_dir, block, &json_text, &mut latin1);
    }
    write_mod_rs(&out_dir, &latin1);
}

fn write_mod_rs(out_dir: &Path, latin1: &[Option<String>]) {
    // Generate: per-block modules + a compact bitmap array for fast negative checks.
    let mut mod_lines = String::from("// Auto-generated. Do not edit manually.\n");
    mod_lines.push_str("// Each block: 256 code points. BITMAP[block][byte] bit set => mapping present.\n");

    // Build bitmap vector
    let mut bitmaps: Vec<[u8; 32]> = Vec::new(); // 256 bits = 32 bytes
    for block in 0u32..0x110u32 {
        let fname = format!("{:02x}.rs", block);
        if out_dir.join(&fname).exists() {
            // Extract keys from the generated file to set bits (parse lines with '=>')
            let content = fs::read_to_string(out_dir.join(&fname)).expect("read block file");
            let mut bits = [0u8; 32];
            for line in content.lines() {
                if let Some(pos) = line.find("=>") {
                    let left = line[..pos].trim();
                    if let Some(cp_str) = left.strip_suffix("u32") {
                        let cp_trim = cp_str.trim();
                        if let Ok(cp) = cp_trim.parse::<u32>() {
                            let low = cp & 0xFF; // position inside block
                            let byte = (low / 8) as usize;
                            let bit = (low % 8) as u8;
                            bits[byte] |= 1 << bit;
                        }
                    }
                }
            }
            bitmaps.push(bits);
            mod_lines.push_str(&format!("pub mod m{:02x} {{ include!(\"./{:02x}.rs\"); }}\n", block, block));
        } else {
            bitmaps.push([0u8; 32]);
        }
    }

    // Emit bitmap static
    mod_lines.push_str("pub static BLOCK_BITMAPS: &[[u8;32]] = &[\n");
    for bits in &bitmaps {
        mod_lines.push_str("    [");
        for (i, b) in bits.iter().enumerate() {
            if i > 0 { mod_lines.push(','); }
            mod_lines.push_str(&format!("0x{:02x}", b));
        }
        mod_lines.push_str("],\n");
    }
    mod_lines.push_str("];\n");

    // Dispatcher using bitmap fast negative check
    mod_lines.push_str(r#"pub fn lookup(cp: u32) -> Option<&'static str> {
    let block = (cp >> 8) as usize;
    if block >= BLOCK_BITMAPS.len() { return None; }
    let idx = (cp & 0xFF) as u32;
    let b = BLOCK_BITMAPS[block];
    let byte = (idx / 8) as usize;
    let bit = (idx % 8) as u8;
    if (b[byte] & (1 << bit)) == 0 { return None; }
    match block {
"#);

    for block in 0u32..0x110u32 {
        let fname = format!("{:02x}.rs", block);
        if out_dir.join(&fname).exists() {
            mod_lines.push_str(&format!(
                "        0x{:x} => m{:02x}::BLOCK_{:02X}.get(&cp).copied(),\n",
                block, block, block
            ));
        }
    }
    mod_lines.push_str("        _ => None,\n    }\n}\n");

    // Emit direct table for 0x00-0xFF (empty string means no mapping / removed)
    mod_lines.push_str("pub static MAP_0_255: [&'static str; 256] = [\n");
    for cp in 0u32..256u32 {
        if cp < 0x80 { // ASCII: identity, represent as empty => no override
            mod_lines.push_str("    \"\",\n");
            continue;
        }
        if let Some(Some(lit)) = latin1.get(cp as usize).map(|o| o.as_ref()) {
            mod_lines.push_str("    ");
            mod_lines.push_str(lit);
            mod_lines.push_str(",\n");
        } else {
            mod_lines.push_str("    \"\",\n");
        }
    }
    mod_lines.push_str("];// MAP_0_255\n");

    mod_lines.push_str("#[inline] pub fn lookup_0_255(cp: u32) -> Option<&'static str> { let s = MAP_0_255[cp as usize]; if s.is_empty() { None } else { Some(s) } }\n");

    fs::write(out_dir.join("mod.rs"), mod_lines).expect("failed to write unidecode_table/mod.rs");
}
fn write_block(out_dir: &Path, block: u32, json_text: &str, latin1: &mut [Option<String>]) {
    let v: serde_json::Value = match serde_json::from_str(json_text) {
        Ok(v) => v,
        Err(e) => panic!("invalid json from python for block {:02x}: {}", block, e),
    };
    let obj = match v.as_object() {
        Some(o) => o,
        None => return,
    };
    if obj.is_empty() {
    let _ = fs::remove_file(out_dir.join(format!("{:02x}.rs", block))); // remove stale empty block file if present
        return;
    }

    let mut entries = String::new();
    for (k, val) in obj {
        let cp: u32 = k.parse().expect("invalid codepoint key from python json");
        let s = val.as_str().expect("expected string value in unidecode json");
        let s_escaped = format!("{:?}", s); // Valid Rust string literal
        entries.push_str(&format!("    {}u32 => {},\n", cp, s_escaped));
        if cp < 256 && cp >= 0x80 { // Only store non-ASCII transliterations in latin1 table
            latin1[cp as usize] = Some(s_escaped);
        }
    }

    let content = format!(
        r#"use phf::phf_map;

pub static BLOCK_{:02X}: phf::Map<u32, &'static str> = phf_map!{{
{}
}};
"#,
        block, entries
    );

    let fname = out_dir.join(format!("{:02x}.rs", block));
    fs::write(&fname, content).expect("failed to write block file");
}

