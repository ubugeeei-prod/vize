#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-cold-pair/replay-build.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
target="$RUNNER_TEMP/replay-target"
args=(--profile ci-opt -p vize --bin vize)
sides=(base head)
case "$VARIANT" in
  alloc) args+=(--features profiling) ;;
  perf)
    sides=(head)
    export CARGO_PROFILE_CI_OPT_DEBUG=line-tables-only
    export CARGO_PROFILE_CI_OPT_STRIP=none
    args+=(--config 'profile.ci-opt.package.vize.strip="none"')
    export RUSTFLAGS="${RUSTFLAGS:-} -C force-frame-pointers=yes"
    export CFLAGS="${CFLAGS:-} -fno-omit-frame-pointer"
    ;;
esac
for side in "${sides[@]}"; do
  sha=$BASE; [[ $side == head ]] && sha=$HEAD
  git fetch --no-tags --depth=1 "https://github.com/$GITHUB_REPOSITORY.git" "$sha"
  git worktree add --detach "$RUNNER_TEMP/src-$side" "$sha"
  mkdir -p "replay-bin/$VARIANT/$side"
  (cd "$RUNNER_TEMP/src-$side" && rustc -Vv) > "replay-bin/$VARIANT/$side-rustc.txt"
  # One target directory: the second build recompiles only what differs.
  (cd "$RUNNER_TEMP/src-$side" && CARGO_TARGET_DIR="$target" cargo build "${args[@]}")
  cp "$target/ci-opt/vize" "replay-bin/$VARIANT/$side/"
done
ls -l replay-bin/*/*
