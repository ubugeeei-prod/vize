#!/usr/bin/env bash
set -euo pipefail
[[ "$BASE_SHA" =~ ^[0-9a-f]{40}$ && "$HEAD_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$(git rev-parse HEAD)" == "$HEAD_SHA" ]]
[[ "$(git merge-base "$BASE_SHA" "$HEAD_SHA")" == "$BASE_SHA" ]]
capture="$RUNNER_TEMP/asset-import-bindings"
baseline="$RUNNER_TEMP/asset-import-bindings-base"
mkdir -p "$capture"
git worktree add --detach "$baseline" "$BASE_SHA"
observer=crates/vize_atelier_sfc/examples/asset_import_names_profile.rs
cp "$observer" "$baseline/$observer"
cmp "$observer" "$baseline/$observer"
git -C "$baseline" diff --exit-code -- crates/vize_atelier_sfc/src Cargo.lock
git diff --exit-code "$BASE_SHA" "$HEAD_SHA" -- Cargo.lock pnpm-lock.yaml
rustc -Vv > "$capture/rustc.txt"
node --version > "$capture/node.txt"
sha256sum Cargo.lock pnpm-lock.yaml "$observer" tests/tooling/asset-import-binding-source.mjs > "$capture/inputs.sha256"
for epoch in head base; do
  if [[ "$epoch" == base ]]; then source="$baseline"; sha="$BASE_SHA"; else source="$GITHUB_WORKSPACE"; sha="$HEAD_SHA"; fi
  target="$RUNNER_TEMP/asset-bindings-targets/$sha"
  token="$(node "$VIZE_CI_FILES/build-immutable-before-and-head-producers-in-separate-source-targets.cjs")"
  printf '%s\n' "$sha $target $token" >> "$capture/source-target-token.txt"
  (cd "$source" && CARGO_TARGET_DIR="$target" VIZE_NATIVE_BUILD_TOKEN="$token" cargo build --locked -p vize_vitrine --features napi,legacy --profile ci --message-format=json-render-diagnostics) > "$capture/$epoch-native-build.jsonl" 2> "$capture/$epoch-native-build.stderr"
  cp "$target/ci/libvize_vitrine.so" "$capture/$epoch.node"
  (cd "$source" && CARGO_TARGET_DIR="$target" cargo build --locked -p vize_atelier_sfc --example asset_import_names_profile --profile ci-opt --message-format=json-render-diagnostics) > "$capture/$epoch-observer-build.jsonl" 2> "$capture/$epoch-observer-build.stderr"
  cp "$target/ci-opt/examples/asset_import_names_profile" "$capture/$epoch-observer"
  SOURCE_SHA="$sha" "$capture/$epoch-observer" > "$capture/$epoch-profile.json" 2> "$capture/$epoch-profile.stderr"
  SOURCE_SHA="$sha" node tests/tooling/asset-import-binding-source.mjs "$epoch" "$capture/$epoch.node" "$capture" > "$capture/$epoch-native.stdout" 2> "$capture/$epoch-native.stderr"
done
CARGO_TARGET_DIR="$RUNNER_TEMP/asset-bindings-targets/$HEAD_SHA" cargo test --locked -p vize_atelier_sfc --lib bundler::tests --profile ci -- --nocapture > "$capture/unit.stdout" 2> "$capture/unit.stderr"
CARGO_TARGET_DIR="$RUNNER_TEMP/asset-bindings-targets/$HEAD_SHA" cargo test --locked -p vize_croquis --lib drawer::helpers::v_for --profile ci -- --nocapture > "$capture/for-helper-unit.stdout" 2> "$capture/for-helper-unit.stderr"
sha256sum "$capture/"*.node "$capture/"*-observer > "$capture/executables.sha256"
