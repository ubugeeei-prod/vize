#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-compare/scripts/replay-build-the-npm-launcher-and-the-native-binding-for-base-and-head.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
for side in base head; do
  sha=$BASE; [[ $side == head ]] && sha=$HEAD
  git fetch --no-tags --depth=1 "https://github.com/$GITHUB_REPOSITORY.git" "$sha"
  git worktree add --detach "$RUNNER_TEMP/npm-$side" "$sha"
  (cd "$RUNNER_TEMP/npm-$side" && vp install --frozen-lockfile --prefer-offline --filter './npm/cli...' --filter './npm/native...' > /dev/null && cargo --version)
done
pids=()
for side in base head; do
  (cd "$RUNNER_TEMP/npm-$side" && vp run --workspace-root build:native > "$RUNNER_TEMP/native-$side.log" 2>&1) &
  pids+=("$!")
done
status=0
for pid in "${pids[@]}"; do wait "$pid" || status=1; done
for side in base head; do tail -n 5 "$RUNNER_TEMP/native-$side.log"; done
[[ "$status" == 0 ]]
for side in base head; do (cd "$RUNNER_TEMP/npm-$side" && vp run --filter './npm/cli' build > /dev/null); done
