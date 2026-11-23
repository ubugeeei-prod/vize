#!/usr/bin/env bash
set -euo pipefail
[[ "$BASE_SHA" =~ ^[0-9a-f]{40}$ ]]
baseline="$RUNNER_TEMP/jsx-origin-base"
observer=crates/vize_atelier_jsx/tests/backend_codegen_profile.rs
git worktree add --detach "$baseline" "$BASE_SHA"
cp "$observer" "$baseline/$observer"
cmp "$observer" "$baseline/$observer"
git -C "$baseline" diff --exit-code -- crates/*/src
(cd "$baseline" && rustc -Vv) > "$RUNNER_TEMP/jsx-backend-origins/base-rustc.txt"
rustc -Vv > "$RUNNER_TEMP/jsx-backend-origins/head-rustc.txt"
(cd "$baseline" && CARGO_TARGET_DIR="$RUNNER_TEMP/jsx-origin-base-target" VIZE_JSX_DEFAULT_PROFILE_CAPTURE="$RUNNER_TEMP/jsx-backend-origins/base-profile.json" cargo test --locked --profile ci -p vize_atelier_jsx --test backend_codegen_profile -- --nocapture)
VIZE_JSX_DEFAULT_PROFILE_CAPTURE="$RUNNER_TEMP/jsx-backend-origins/head-profile.json" cargo test --locked --profile ci -p vize_atelier_jsx --test backend_codegen_profile -- --nocapture
git rev-parse HEAD HEAD^{tree} > "$RUNNER_TEMP/jsx-backend-origins/head-source.txt"
git -C "$baseline" rev-parse HEAD HEAD^{tree} > "$RUNNER_TEMP/jsx-backend-origins/base-source.txt"
sha256sum "$observer" "$baseline/$observer" > "$RUNNER_TEMP/jsx-backend-origins/observer.sha256"
