#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-compare/scripts/build-both-commits-2.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
target="$RUNNER_TEMP/pair-target"
for side in base head; do
  sha=$BASE; [[ $side == head ]] && sha=$HEAD
  git fetch --no-tags --depth=1 "https://github.com/$GITHUB_REPOSITORY.git" "$sha"
  git worktree add --detach "$RUNNER_TEMP/src-$side" "$sha"
  # One target directory: the second build recompiles only what differs.
  (cd "$RUNNER_TEMP/src-$side" && CARGO_TARGET_DIR="$target" cargo build --profile ci-opt -p vize --bin vize)
  mkdir -p "pair-bin/$side"
  cp "$target/ci-opt/vize.exe" "pair-bin/$side/"
done
ls -l pair-bin/*
