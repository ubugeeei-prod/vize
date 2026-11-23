#!/usr/bin/env bash
source_sha="$EXPECTED_SOURCE_SHA"
source_root="$GITHUB_WORKSPACE"
if [[ "$ROLE" == baseline ]]; then
  source_sha="$BASELINE_SHA"
  source_root="$RUNNER_TEMP/doctor-baseline-source"
  git worktree add --detach "$source_root" "$source_sha"
  cp crates/vize/tests/doctor_lint_ssr_context_cli.rs "$source_root/crates/vize/tests/"
  echo 'mod doctor_lint_ssr_context_cli;' >> "$source_root/crates/vize/tests/main.rs"
fi
target_root="$RUNNER_TEMP/doctor-ssr-$source_sha-$ROLE-target"
test ! -e "$target_root"
{
  echo "SOURCE_SHA=$source_sha"
  echo "SOURCE_ROOT=$source_root"
  echo "CARGO_TARGET_DIR=$target_root"
  echo "VIZE_SSR_CONTEXT_ARTIFACTS=$RUNNER_TEMP/doctor-ssr-packets"
} >> "$GITHUB_ENV"
mkdir -p "$RUNNER_TEMP/doctor-ssr-packets"
printf '%s\n' "$source_sha" > "$RUNNER_TEMP/doctor-ssr-packets/source-sha.txt"
git -C "$source_root" status --porcelain > "$RUNNER_TEMP/doctor-ssr-packets/source-status.txt"
