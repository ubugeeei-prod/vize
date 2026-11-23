#!/usr/bin/env bash
source_root="$GITHUB_WORKSPACE"
source_sha="$HEAD_SHA"
if [ "$ROLE" = baseline ]; then
  source_sha="$BASE_SHA"
  source_root="$RUNNER_TEMP/native-diagnostics-baseline-$source_sha"
  git worktree add --detach "$source_root" "$source_sha"
  ln -s "$GITHUB_WORKSPACE/node_modules" "$source_root/node_modules"
  ln -s "$GITHUB_WORKSPACE/tests/node_modules" "$source_root/tests/node_modules"
fi
target_dir="$RUNNER_TEMP/native-diagnostics-target-$ROLE-$source_sha"
test ! -e "$target_dir"
export CARGO_TARGET_DIR="$target_dir"
cd "$source_root"
cargo build --locked --profile ci -p vize
mkdir -p target/ci
cp "$target_dir/ci/vize" target/ci/vize
node tests/differential/build-receipt.ts
echo "VIZE_WARMUP_ROLE=$ROLE" >> "$GITHUB_ENV"
echo "VIZE_WARMUP_SOURCE_ROOT=$source_root" >> "$GITHUB_ENV"
echo "CARGO_TARGET_DIR=$target_dir" >> "$GITHUB_ENV"
echo "VIZE_LSP_BIN=$source_root/target/ci/vize" >> "$GITHUB_ENV"
