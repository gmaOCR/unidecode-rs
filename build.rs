use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Build-time script: interroge la bibliothèque Python Unidecode bloc par bloc (256 codepoints)
    // et génère des fichiers `xx.rs` (où xx est l'offset du bloc en hex) dans src/unidecode_table/.
    // Un `mod.rs` est ensuite écrit avec des modules mXX qui `include!` les fichiers.

    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out_dir = crate_root.join("src").join("unidecode_table");

    if !out_dir.exists() {
        fs::create_dir_all(&out_dir).expect("failed to create src/unidecode_table");
    }

    let python = env::var("PYTHON").unwrap_or_else(|_| "python3".to_string());

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
            // try to install Unidecode and retry once
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
        write_block(&out_dir, block, &json_text);
    }
    write_mod_rs(&out_dir);
}

fn write_mod_rs(out_dir: &Path) {
    // Génère les modules mXX et un dispatcher vers BLOCK_XX
    let mut mod_lines = String::from("// Auto-generated. Do not edit manually.\n");
    let mut dispatcher = String::from("pub fn lookup(cp: u32) -> Option<&'static str> {\n    match cp >> 8 {\n");

    for block in 0u32..0x110u32 {
        let fname = format!("{:02x}.rs", block);
        if out_dir.join(&fname).exists() {
            mod_lines.push_str(&format!("pub mod m{:02x} {{ include!(\"./{:02x}.rs\"); }}\n", block, block));
            dispatcher.push_str(&format!(
                "        0x{:x} => m{:02x}::BLOCK_{:02X}.get(&cp).copied(),\n",
                block, block, block
            ));
        }
    }

    dispatcher.push_str("        _ => None,\n    }\n}\n");
    let content = format!("{}\n{}", mod_lines, dispatcher);
    fs::write(out_dir.join("mod.rs"), content).expect("failed to write unidecode_table/mod.rs");
}

fn write_block(out_dir: &Path, block: u32, json_text: &str) {
    let v: serde_json::Value = match serde_json::from_str(json_text) {
        Ok(v) => v,
        Err(e) => panic!("invalid json from python for block {:02x}: {}", block, e),
    };
    let obj = match v.as_object() {
        Some(o) => o,
        None => return,
    };
    if obj.is_empty() {
        let _ = fs::remove_file(out_dir.join(format!("{:02x}.rs", block))); // nettoyer ancien
        return;
    }

    let mut entries = String::new();
    for (k, val) in obj {
        let cp: u32 = k.parse().expect("invalid codepoint key from python json");
        let s = val.as_str().expect("expected string value in unidecode json");
        let s_escaped = format!("{:?}", s);
    entries.push_str(&format!("    {}u32 => {},\n", cp, s_escaped));
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

