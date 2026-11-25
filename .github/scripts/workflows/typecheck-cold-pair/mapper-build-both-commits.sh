#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-cold-pair/mapper-build-both-commits.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
target="$RUNNER_TEMP/pair-target"
mkdir -p pair-bin
for side in base head; do
  sha=$BASE; [[ $side == head ]] && sha=$HEAD
  git fetch --no-tags --depth=1 "https://github.com/$GITHUB_REPOSITORY.git" "$sha"
  git worktree add --detach "$RUNNER_TEMP/src-$side" "$sha"
  (cd "$RUNNER_TEMP/src-$side" && rustc -Vv) > "pair-bin/$side-rustc.txt"
  # One target directory: the second build recompiles only what differs.
  (cd "$RUNNER_TEMP/src-$side" && CARGO_TARGET_DIR="$target" cargo build --profile ci-opt -p vize --bin vize)
  mkdir -p "pair-bin/$side"
  cp "$target/ci-opt/$PAIR_EXE" "pair-bin/$side/"
done
ls -l pair-bin/*
