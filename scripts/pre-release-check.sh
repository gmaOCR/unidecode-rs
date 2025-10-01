#!/bin/bash
# Script de pré-publication pour unidecode-rs
# Vérifie que tout est prêt pour la publication

set -e

echo "🔍 Vérification pré-publication unidecode-rs"
echo "=============================================="
echo

# 1. Format
echo "📝 Vérification du format..."
cargo fmt --all --check
echo "✅ Format OK"
echo

# 2. Lint
echo "🔍 Vérification lint (clippy)..."
cargo clippy --all-features -- -D warnings
echo "✅ Lint OK"
echo

# 3. Tests Rust
echo "🧪 Tests Rust..."
cargo test --release --all-features
echo "✅ Tests Rust OK"
echo

# 4. Build release
echo "🔨 Build release..."
cargo build --release --features python
echo "✅ Build OK"
echo

# 5. Benchmarks
echo "⚡ Benchmarks Criterion..."
cargo bench --bench unidecode_bench 2>&1 | grep -E "(time:|Benchmarking)" | tail -20
echo "✅ Benchmarks OK"
echo

# 6. Tests Python (si venv existe)
if [ -d ".venv" ]; then
    echo "🐍 Tests Python (extension)..."
    source .venv/bin/activate
    
    # Rebuild extension
    maturin develop --release --features python --quiet
    
    # Vérifie l'import
    python -c "import unidecode_rs; print(f'Version: {unidecode_rs.__version__}')"
    python -c "import unidecode_rs; assert unidecode_rs.unidecode('Café') == 'Cafe'"
    
    echo "✅ Extension Python OK"
    echo
    
    # Benchmark vs Python
    echo "📊 Benchmark vs Python..."
    python scripts/bench_compare.py 2>&1 | grep -E "(python time|rust time|equal)"
    echo "✅ Benchmark OK"
    echo
    
    deactivate
else
    echo "⚠️  Pas de venv, tests Python ignorés"
    echo
fi

echo "=============================================="
echo "✨ Tout est prêt pour la publication ! ✨"
echo
echo "Prochaines étapes :"
echo "1. Mettre à jour la version dans Cargo.toml (actuellement $(grep '^version' Cargo.toml | head -1))"
echo "2. Finaliser CHANGELOG.md (déplacer Unreleased → version)"
echo "3. Commit : git commit -am 'Release vX.Y.Z'"
echo "4. Tag : git tag -a vX.Y.Z -m 'Version X.Y.Z'"
echo "5. Push : git push && git push --tags"
echo "6. Publier crate : cargo publish"
echo "7. Publier wheel : maturin publish --features python"
