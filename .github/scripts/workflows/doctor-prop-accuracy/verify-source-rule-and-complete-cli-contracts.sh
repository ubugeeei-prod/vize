#!/usr/bin/env bash
cd "$SOURCE_ROOT"
cargo test --locked --profile ci -p vize_croquis_cf --lib
cargo test --locked --profile ci -p vize --lib commands::doctor::tests::
/usr/bin/time -v cargo test --locked --profile ci -p vize --test integration doctor_ -- --nocapture > "$VIZE_PROP_ACCURACY_ARTIFACTS/test.stdout" 2> "$VIZE_PROP_ACCURACY_ARTIFACTS/test.stderr"
cat "$VIZE_PROP_ACCURACY_ARTIFACTS/test.stdout"
cargo clippy --locked --profile ci -p vize_croquis_cf --lib -- -D warnings
