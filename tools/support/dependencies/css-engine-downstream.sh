#!/usr/bin/env bash
# Keep nine original controls and add three namespace-only migration controls.
set -euo pipefail
css_identity_root="$PWD"
css_identity_evidence="${RUNNER_TEMP:?}/css-engine-identity"
mkdir -p "$css_identity_evidence"
sha256sum tests/external-consumers/css-engine-identity/src/main.rs tests/external-consumers/css-engine-identity/typed-visitors.rs tests/external-consumers/css-engine-identity/trait-implementation.rs > "$css_identity_evidence/source.sha256"
sha256sum tests/external-consumers/css-engine-identity/patina-provider-new-raw.rs tests/external-consumers/css-engine-identity/patina-provider-typed-visitors.rs tests/external-consumers/css-engine-identity/patina-provider-trait-implementation.rs > "$css_identity_evidence/migration-source.sha256"
git rev-parse HEAD > "$css_identity_evidence/source-head.txt"
rustc -Vv > "$css_identity_evidence/rustc.txt"
cp tests/external-consumers/css-engine-identity/src/main.rs "$css_identity_evidence/consumer.rs"
cp tests/external-consumers/css-engine-identity/typed-visitors.rs "$css_identity_evidence/typed-visitors.rs"
cp tests/external-consumers/css-engine-identity/trait-implementation.rs "$css_identity_evidence/trait-implementation.rs"
for css_identity_contract in new-raw typed-visitors trait-implementation; do
  cp "tests/external-consumers/css-engine-identity/patina-provider-$css_identity_contract.rs" "$css_identity_evidence/patina-provider-$css_identity_contract.rs"
done
git worktree add --detach "$css_identity_evidence/baseline" "${CSS_ENGINE_BASE_SHA:?}"
git -C "$css_identity_evidence/baseline" rev-parse HEAD > "$css_identity_evidence/baseline-head.txt"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$css_identity_root/target/css-engine-identity}"
export CARGO_PROFILE_DEV_DEBUG=0
for css_identity_plane in baseline-original candidate-original candidate-fork candidate-patina; do
for css_identity_contract in new-raw typed-visitors trait-implementation; do
  css_identity_directory="$css_identity_evidence/$css_identity_plane-$css_identity_contract"
  mkdir -p "$css_identity_directory/src"
  if [[ "$css_identity_contract" == new-raw ]]; then
    css_identity_source=tests/external-consumers/css-engine-identity/src/main.rs
  else
    css_identity_source="tests/external-consumers/css-engine-identity/$css_identity_contract.rs"
  fi
  if [[ "$css_identity_plane" == candidate-patina ]]; then
    css_identity_source="tests/external-consumers/css-engine-identity/patina-provider-$css_identity_contract.rs"
  fi
  cp "$css_identity_source" "$css_identity_directory/src/main.rs"
  if [[ "$css_identity_plane" == baseline-original ]]; then
    css_identity_patina="$css_identity_evidence/baseline/crates/vize_patina"
  else
    css_identity_patina="$css_identity_root/crates/vize_patina"
  fi
  {
    printf '[package]\nname = "css-engine-identity"\nversion = "0.0.0"\nedition = "2021"\npublish = false\n[workspace]\n[dependencies]\n'
    printf 'vize_patina = { path = "%s" }\n' "$css_identity_patina"
    if [[ "$css_identity_plane" == candidate-fork ]]; then
      printf 'lightningcss = { package = "vize_lightningcss", path = "%s/vendor/lightningcss" }\n' "$css_identity_root"
    else
      printf 'lightningcss = "=1.0.0-alpha.72"\n'
    fi
  } > "$css_identity_directory/Cargo.toml"
  css_identity_status=0
  cargo run --manifest-path "$css_identity_directory/Cargo.toml" > "$css_identity_directory/stdout.log" 2> "$css_identity_directory/stderr.log" || css_identity_status=$?
  printf '%s\n' "$css_identity_status" > "$css_identity_directory/exit.txt"
  cargo metadata --locked --manifest-path "$css_identity_directory/Cargo.toml" --format-version 1 > "$css_identity_directory/metadata.json"
  cat "$css_identity_directory/stderr.log"
  if [[ "$css_identity_plane" == candidate-original ]]; then
    [[ "$css_identity_status" == 101 ]]
    case "$css_identity_contract" in
      new-raw)
        rg 'error\[E0308\]' "$css_identity_directory/stderr.log"
        rg 'ParsedStyleSheet::new|parsed.raw' "$css_identity_directory/stderr.log"
        ;;
      typed-visitors)
        rg 'error\[E0631\]' "$css_identity_directory/stderr.log"
        rg 'for_each_style_rule' "$css_identity_directory/stderr.log"
        rg 'for_each_font_face_rule' "$css_identity_directory/stderr.log"
        ;;
      trait-implementation)
        rg 'error\[E0053\]' "$css_identity_directory/stderr.log"
        rg 'check.*incompatible type for trait|incompatible type for trait.*check' "$css_identity_directory/stderr.log"
        ;;
    esac
  else
    [[ "$css_identity_status" == 0 ]]
  fi
done
done
printf 'three whole authored contracts x four dependency planes = twelve\nbaseline original: compatible\ncandidate original: incompatible nominal type\ncandidate fork: compatible\ncandidate Patina namespace: documented migration, not transparent compatibility\nsource-only; no installed or registry qualification\n' > "$css_identity_evidence/report.txt"
