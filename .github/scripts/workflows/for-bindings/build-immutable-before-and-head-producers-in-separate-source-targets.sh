#!/usr/bin/env bash
set -euo pipefail
[[ "$BASE_SHA" =~ ^[0-9a-f]{40}$ && "$HEAD_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$(git rev-parse HEAD)" == "$HEAD_SHA" ]]
[[ "$(git merge-base "$BASE_SHA" "$HEAD_SHA")" == "$BASE_SHA" ]]
capture="$RUNNER_TEMP/for-bindings"
baseline="$RUNNER_TEMP/for-bindings-base"
mkdir -p "$capture"
git worktree add --detach "$baseline" "$BASE_SHA"
observer=crates/vize_atelier_sfc/examples/for_binding_profile.rs
cp "$observer" "$baseline/$observer"
cmp "$observer" "$baseline/$observer"
git -C "$baseline" diff --exit-code -- crates/vize_atelier_sfc/src Cargo.lock
git diff --exit-code "$BASE_SHA" "$HEAD_SHA" -- crates/vize_atelier_sfc/src
git diff --exit-code "$BASE_SHA" "$HEAD_SHA" -- Cargo.lock pnpm-lock.yaml
node --version > "$capture/node.txt"
sha256sum Cargo.lock pnpm-lock.yaml "$observer" tests/tooling/for-binding-source.mjs tests/tooling/for-key-literal-source.mjs tests/tooling/for-binding-verify.mjs > "$capture/inputs.sha256"
for epoch in head base; do
  if [[ "$epoch" == base ]]; then source="$baseline"; sha="$BASE_SHA"; else source="$GITHUB_WORKSPACE"; sha="$HEAD_SHA"; fi
  (cd "$source" && rustc -Vv) > "$capture/$epoch-rustc.txt"
  target="$RUNNER_TEMP/for-bindings-targets/$sha"
  token="$(node "$VIZE_CI_FILES/build-immutable-before-and-head-producers-in-separate-source-targets.cjs")"
  printf '%s\n' "$sha $target $token" >> "$capture/source-target-token.txt"
  (cd "$source" && CARGO_TARGET_DIR="$target" VIZE_NATIVE_BUILD_TOKEN="$token" cargo build --locked -p vize_vitrine --features napi,legacy --profile ci --message-format=json-render-diagnostics) > "$capture/$epoch-native-build.jsonl" 2> "$capture/$epoch-native-build.stderr"
  cp "$target/ci/libvize_vitrine.so" "$capture/$epoch.node"
  (cd "$source" && CARGO_TARGET_DIR="$target" cargo build --locked -p vize_atelier_sfc --example for_binding_profile --profile ci-opt --message-format=json-render-diagnostics) > "$capture/$epoch-observer-build.jsonl" 2> "$capture/$epoch-observer-build.stderr"
  cp "$target/ci-opt/examples/for_binding_profile" "$capture/$epoch-observer"
  SOURCE_SHA="$sha" "$capture/$epoch-observer" > "$capture/$epoch-profile.json" 2> "$capture/$epoch-profile.stderr"
  SOURCE_SHA="$sha" node tests/tooling/for-binding-source.mjs "$epoch" "$capture/$epoch.node" "$capture" > "$capture/$epoch-native.stdout" 2> "$capture/$epoch-native.stderr"
done
CARGO_TARGET_DIR="$RUNNER_TEMP/for-bindings-targets/$HEAD_SHA" cargo test --locked -p vize_atelier_core --lib v_for --profile ci -- --nocapture > "$capture/unit.stdout" 2> "$capture/unit.stderr"
CARGO_TARGET_DIR="$RUNNER_TEMP/for-bindings-targets/$HEAD_SHA" cargo test --locked -p vize_l1_to_l2 --lib lower::forop --profile ci -- --nocapture > "$capture/native-for-unit.stdout" 2> "$capture/native-for-unit.stderr"
CARGO_TARGET_DIR="$RUNNER_TEMP/for-bindings-targets/$HEAD_SHA" cargo test --locked -p vize_atelier_vapor --lib generate::operations::for_loop --profile ci -- --nocapture > "$capture/for-key-unit.stdout" 2> "$capture/for-key-unit.stderr"
for log in unit native-for-unit for-key-unit; do
  grep -Eq 'test result: ok\. [1-9][0-9]* passed; 0 failed; 0 ignored;' "$capture/$log.stdout"
done
sha256sum "$capture/"*.node "$capture/"*-observer > "$capture/executables.sha256"
