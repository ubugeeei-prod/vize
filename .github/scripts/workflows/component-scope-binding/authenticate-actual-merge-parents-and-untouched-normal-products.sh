#!/usr/bin/env bash
set -euo pipefail
[[ "$(git rev-parse HEAD^2)" == "$PR_HEAD_SHA" ]]
export PRODUCT_BASE_SHA=$(git rev-parse HEAD^1)
export PRODUCT_HEAD_SHA=$(git rev-parse HEAD)
export EVIDENCE="$RUNNER_TEMP/component-scope-binding-evidence"
mkdir "$EVIDENCE"
git worktree add --detach "$RUNNER_TEMP/component-scope-binding-base" "$PRODUCT_BASE_SHA"
observer=tools/benchmarks/crates/vize/tests/component_scope_binding_profile.rs
cp "$observer" "$RUNNER_TEMP/component-scope-binding-base/$observer"
node "$VIZE_CI_FILES/../shared/component-provenance.ts"
for role in base head; do
  source_root="$GITHUB_WORKSPACE"
  if [[ "$role" == base ]]; then source_root="$RUNNER_TEMP/component-scope-binding-base"; fi
  git -C "$source_root" diff --exit-code
  git -C "$source_root" rev-parse HEAD > "$EVIDENCE/$role.source"
  (cd "$source_root" && rustc -Vv) > "$EVIDENCE/$role-rustc.txt"
  (cd "$source_root" && CARGO_TARGET_DIR="$RUNNER_TEMP/component-$role-normal-target" cargo build --locked --profile ci-opt -p vize --bin vize --message-format=json) > "$EVIDENCE/$role-normal-build.jsonl"
  node "$VIZE_CI_FILES/../shared/profile-build.ts" vize "$role"
done
node "$VIZE_CI_FILES/authenticate-actual-merge-parents-and-untouched-normal-products.ts"
uname -a > "$EVIDENCE/kernel.txt"
lscpu > "$EVIDENCE/cpu.txt"
