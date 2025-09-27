# Coverage guidance for unidecode-rs

This crate exposes both a Rust API and optional Python bindings via PyO3.
We collect two kinds of coverage:

1. Rust line coverage (cargo-llvm-cov -> lcov.info)
2. Python test coverage (pytest + pytest-cov -> coverage.xml)

## Rust coverage

Chemins fournis via alias cargo (voir `.cargo/config.toml`) pour homogénéité avec `slugify-rs`.

### Rapide (HTML interactif)
```bash
cargo coverage
```
Ouvre un rapport HTML (exclut binaires selon regex d'ignore).

### Format lcov pour CI / Codecov
```bash
cargo coverage-lcov
```
Produit `lcov.info` à la racine.

### Manual (équivalent bas niveau)
```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov # first time only
cargo llvm-cov --package unidecode-rs --lcov --output-path lcov.info \
	--ignore-filename-regex '/usr/src|/rustc-|src/bin/.*|bin/.*'
```

## Python coverage

```bash
python -m venv .venv
. .venv/bin/activate
pip install --upgrade pip maturin pytest pytest-cov coverage
maturin develop --release --features python
pytest --cov=. --cov-report=xml:coverage.xml --cov-report=term-missing
```

Artifacts:
- `lcov.info`  -> upload with Codecov flag `rust`
- `coverage.xml` -> upload with Codecov flag `python`

## Combined view
Codecov fusionne les uploads différenciés par flags (`rust`, `python`).
Pour repo privé définir `CODECOV_TOKEN`; inutile pour public.

Pipeline type (local):
```bash
./scripts/coverage.sh  # (après création du script, voir section suivante)
```

## Adding new tests
- Rust: prefer small deterministic unit tests in `tests/` or `#[cfg(test)]` modules.
- Python: add integration / golden tests exercising edge code points.

## Next optimization stages
Les optimisations futures (skip bits, tables denses, vectorisation potentielle) doivent :
- ajouter un bench Criterion dédié (régression perf) ;
- enrichir les golden tests pour verrouiller la sortie ;
- maintenir >50% couverture lignes cœur, viser >90% pour `lib.rs` (actuel ~98%).

## Planned helper script
Un script `scripts/coverage.sh` (non encore ajouté) pourra enchaîner :
1. Installation venv Python + build extension
2. Tests Python + `coverage.xml`
3. `cargo coverage-lcov`
4. Affichage résumé lignes / chemins ignorés

Il reflétera la logique déjà adoptée dans `slugify-rs` pour uniformiser.
