#!/usr/bin/env bash
cd "$SOURCE_ROOT"
cargo test --locked --profile ci -p vize_croquis_cf --lib
cargo test --locked --profile ci -p vize --lib commands::project_ssr::tests::
cargo test --locked --profile ci -p vize --lib commands::doctor::tests::
/usr/bin/time -v cargo test --locked --profile ci -p vize --test integration doctor_lint_ssr_context_cli:: -- --nocapture > "$VIZE_SSR_CONTEXT_ARTIFACTS/test.stdout" 2> "$VIZE_SSR_CONTEXT_ARTIFACTS/test.stderr"
cat "$VIZE_SSR_CONTEXT_ARTIFACTS/test.stdout"
cargo clippy --locked --profile ci -p vize --lib -- -D warnings
