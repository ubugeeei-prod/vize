#!/usr/bin/env bash
# Execute one external Rust consumer body against baseline and candidate types.
set -euo pipefail
css_identity_root="$PWD"
css_identity_evidence="${RUNNER_TEMP:?}/css-engine-identity"
mkdir -p "$css_identity_evidence"
sha256sum tests/external-consumers/css-engine-identity/src/main.rs > "$css_identity_evidence/source.sha256"
git rev-parse HEAD > "$css_identity_evidence/source-head.txt"
rustc -Vv > "$css_identity_evidence/rustc.txt"
cp tests/external-consumers/css-engine-identity/src/main.rs "$css_identity_evidence/consumer.rs"
git worktree add --detach "$css_identity_evidence/baseline" "${CSS_ENGINE_BASE_SHA:?}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$css_identity_root/target/css-engine-identity}"
export CARGO_PROFILE_DEV_DEBUG=0
for css_identity_case in baseline-original candidate-original candidate-fork; do
  css_identity_directory="$css_identity_evidence/$css_identity_case"
  mkdir -p "$css_identity_directory/src"
  cp tests/external-consumers/css-engine-identity/src/main.rs "$css_identity_directory/src/main.rs"
  if [[ "$css_identity_case" == baseline-original ]]; then
    css_identity_patina="$css_identity_evidence/baseline/crates/vize_patina"
  else
    css_identity_patina="$css_identity_root/crates/vize_patina"
  fi
  {
    printf '[package]\nname = "css-engine-identity"\nversion = "0.0.0"\nedition = "2021"\npublish = false\n[workspace]\n[dependencies]\n'
    printf 'vize_patina = { path = "%s" }\n' "$css_identity_patina"
    if [[ "$css_identity_case" == candidate-fork ]]; then
      printf 'lightningcss = { package = "vize_lightningcss", path = "%s/vendor/lightningcss" }\n' "$css_identity_root"
    else
      printf 'lightningcss = "=1.0.0-alpha.72"\n'
    fi
  } > "$css_identity_directory/Cargo.toml"
  css_identity_status=0
  cargo run --manifest-path "$css_identity_directory/Cargo.toml" > "$css_identity_directory/stdout.log" 2> "$css_identity_directory/stderr.log" || css_identity_status=$?
  printf '%s\n' "$css_identity_status" > "$css_identity_directory/exit.txt"
  cat "$css_identity_directory/stderr.log"
  if [[ "$css_identity_case" == candidate-original ]]; then
    [[ "$css_identity_status" == 101 ]]
    rg 'error\[E0308\]' "$css_identity_directory/stderr.log"
    rg 'ParsedStyleSheet::new|parsed.raw' "$css_identity_directory/stderr.log"
  else
    [[ "$css_identity_status" == 0 ]]
  fi
done
printf 'baseline original: compatible\ncandidate original: incompatible nominal type\ncandidate fork: compatible\nsource-only; no installed or registry qualification\n' > "$css_identity_evidence/report.txt"
