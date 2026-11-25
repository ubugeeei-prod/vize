#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-compare/scripts/corpus-build-the-native-binding-and-the-vize-package-for-base-and-head.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
roles=(base head)
# An A/A run builds its one SHA once; pair-base links to pair-head.
if [[ "$BASE_SHA" == "$HEAD_SHA" ]]; then roles=(head); fi
for role in "${roles[@]}"; do
  (
    cd "$RUNNER_TEMP/pair-$role"
    vp install --frozen-lockfile --prefer-offline --filter './npm/cli...' --filter './npm/native...'
    # Install each rust-toolchain.toml toolchain before the parallel builds race on it.
    cargo --version
  )
done
pids=()
for role in "${roles[@]}"; do
  (
    cd "$RUNNER_TEMP/pair-$role"
    if [[ "$OWNED_PAIR" == true ]]; then export CARGO_TARGET_DIR="$PWD/target"; fi
    vp run --workspace-root build:native > "$RUNNER_TEMP/build-$role.log" 2>&1
  ) &
  pids+=("$!")
done
status=0
for pid in "${pids[@]}"; do wait "$pid" || status=1; done
for role in "${roles[@]}"; do echo "::group::native build $role"; tail -n 40 "$RUNNER_TEMP/build-$role.log"; echo "::endgroup::"; done
[[ "$status" == 0 ]]
for role in "${roles[@]}"; do
  (cd "$RUNNER_TEMP/pair-$role" && vp run --filter './npm/cli' build)
  git -C "$RUNNER_TEMP/pair-$role" rev-parse HEAD
done
