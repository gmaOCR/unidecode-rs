//! Golden test: vérifie l'équivalence exacte avec la librairie Python Unidecode
//! et affiche une comparaison de performances (Rust vs Python) sur un échantillon.
//!
//! Exécution: `cargo test -- --nocapture golden_equivalence` (python3 requis dans PATH).

use std::process::Command;
use std::time::Instant; // for JSON-safe string embedding

fn call_python_unidecode(s: &str) -> String {
    // Encode la chaîne en JSON pour éviter problèmes d'escapes Python.
    let json_input = serde_json::to_string(s).expect("json encode input");
    // Script Python robuste: capture/ignore bruit pip, toujours émettre exactement UNE ligne JSON.
    let py = r#"import json,sys,subprocess,traceback
def ensure_unidecode():
    try:
        from unidecode import unidecode as u  # type: ignore
        return u
    except Exception:
        try:
            # Supprime sortie interactive de pip: -q
            subprocess.check_call([sys.executable,'-m','pip','install','-q','Unidecode'])
            from unidecode import unidecode as u  # type: ignore
            return u
        except Exception as e:
            print(json.dumps({{'error':'install','detail':str(e)}}))
            sys.exit(0)
u = ensure_unidecode()
stdin_data = sys.stdin.read()
data = json.loads(stdin_data)
try:
    res = u(data)
    print(json.dumps({'ok':res}))
except Exception as e:
    print(json.dumps({'error':'exec','detail':str(e),'trace':traceback.format_exc()}))
"#;
    let mut child = Command::new(std::env::var("PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg("-c")
        .arg(py)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn python");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(json_input.as_bytes()).expect("write stdin");
    }
    let out = child.wait_with_output().expect("python wait");
    if !out.status.success() {
        panic!(
            "python failed (status {:?}):\nSTDERR:{}\nSTDOUT:{}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
    }
    let stdout = String::from_utf8(out.stdout).expect("non utf8 python output");
    let line = stdout.lines().last().unwrap_or("").trim();
    if line.is_empty() {
        panic!(
            "python returned empty output; stderr={} full_stdout={}",
            String::from_utf8_lossy(&out.stderr),
            stdout
        );
    }
    let val: serde_json::Value = serde_json::from_str(line).expect("python emitted invalid JSON");
    if let Some(obj) = val.as_object() {
        if let Some(ok) = obj.get("ok") {
            return ok.as_str().unwrap().to_string();
        }
        if let Some(err) = obj.get("error") {
            panic!(
                "python script error type={:?} detail={:?}",
                err,
                obj.get("detail")
            );
        }
    }
    panic!("unexpected python json structure: {}", line);
}

#[test]
fn golden_equivalence() {
    let samples = [
        "ASCII plain text",
        "déjà vu — français",
        "Добрый день",    // cyrillique
        "こんにちは世界", // japonais
        "中文字符測試",   // chinois
        "Έλληνες φίλοι",  // grec
        "😀 emoji mix 👍 café",
        "한국어 테스트 문자열",
        "ภาษาไทยทดสอบ",
        "हिन्दी परीक्षण वाक्य", // hindi
    ];

    for &s in &samples {
        let py = call_python_unidecode(s);
        let rs = unidecode_rs::unidecode(s);
        assert_eq!(rs, py, "Mismatch for input {:?}", s);
    }
}

#[test]
fn performance_snapshot() {
    // Micro bench simple (non stable) juste indicatif dans la CI.
    let text =
        "déjà vu — Français Русский текст 中文字符測試 한국어 테스트 😀 emoji こんにちは世界 "
            .repeat(2000);

    // Python timing
    let data_json = serde_json::to_string(&text).unwrap();
    let py_code = r#"import time,json,sys,subprocess
def ensure():
    try:
        from unidecode import unidecode as u  # type: ignore
        return u
    except Exception:
        subprocess.check_call([sys.executable,'-m','pip','install','-q','Unidecode'])
        from unidecode import unidecode as u  # type: ignore
        return u
u = ensure()
stdin_data = sys.stdin.read()
data = json.loads(stdin_data)
start = time.time()
for _ in range(5):
    u(data)
print(time.time()-start)"#;
    let py_start = Instant::now();
    let mut bench_child =
        Command::new(std::env::var("PYTHON").unwrap_or_else(|_| "python3".into()))
            .arg("-c")
            .arg(py_code)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("python bench spawn");
    use std::io::Write as _;
    if let Some(mut stdin) = bench_child.stdin.take() {
        stdin.write_all(data_json.as_bytes()).unwrap();
    }
    let py_out = bench_child.wait_with_output().expect("python bench wait");
    assert!(
        py_out.status.success(),
        "python bench stderr: {}",
        String::from_utf8_lossy(&py_out.stderr)
    );
    let py_reported: f64 = String::from_utf8(py_out.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap_or_else(|_| {
            eprintln!("warn: parse python timing fallback using wall clock");
            py_start.elapsed().as_secs_f64()
        });

    // Rust timing
    let rs_start = Instant::now();
    for _ in 0..5 {
        let _ = unidecode_rs::unidecode(&text);
    }
    let rs_time = rs_start.elapsed().as_secs_f64();

    eprintln!(
        "Perf snapshot: python {:.4}s vs rust {:.4}s speedup x{:.2}",
        py_reported,
        rs_time,
        py_reported / rs_time.max(1e-9)
    );
}
