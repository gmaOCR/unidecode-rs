use criterion::{Criterion, black_box, criterion_group, criterion_main};
use unidecode_rs::unidecode;

fn dataset_short() -> &'static str {
    "Café déjà vu — Русский текст 中文 😀 𝔘𝔫𝔦𝔠𝔬𝔡𝔢"
}

fn dataset_medium() -> String {
    let base = "Pchnąć w tę łódź jeża lub ośm skrzyń fig"; // Polish pangram variant
    let mut s = String::with_capacity(4096);
    for _ in 0..128 {
        s.push_str(base);
        s.push(' ');
    }
    s
}

fn dataset_large() -> String {
    // Mix of scripts repeated
    let chunk = "Σὲ γνωρίζω ἀπὸ τὴν κόψη Съешь ещё этих мягких французских булок 😀 中文測試";
    let mut s = String::with_capacity(64 * 1024);
    for _ in 0..512 {
        s.push_str(chunk);
        s.push(' ');
    }
    s
}

fn bench_unidecode(c: &mut Criterion) {
    c.bench_function("short", |b| {
        b.iter(|| unidecode(black_box(dataset_short())))
    });

    let med = dataset_medium();
    c.bench_function("medium", |b| b.iter(|| unidecode(black_box(&med))));

    let large = dataset_large();
    c.bench_function("large", |b| b.iter(|| unidecode(black_box(&large))));
}

criterion_group!(name=unidecode_benches; config=Criterion::default().sample_size(40); targets=bench_unidecode);
criterion_main!(unidecode_benches);
