#!/usr/bin/env bash
source_sha="$EXPECTED_SOURCE_SHA"
source_root="$GITHUB_WORKSPACE"
if [[ "$ROLE" == baseline ]]; then
  source_sha="$BASELINE_SHA"
  source_root="$RUNNER_TEMP/doctor-baseline-source"
  git worktree add --detach "$source_root" "$source_sha"
  cp crates/vize/tests/doctor_prop_accuracy_cli.rs "$source_root/crates/vize/tests/"
  echo 'mod doctor_prop_accuracy_cli;' >> "$source_root/crates/vize/tests/main.rs"
fi
target_root="$RUNNER_TEMP/doctor-props-$source_sha-$ROLE-target"
test ! -e "$target_root"
echo "SOURCE_SHA=$source_sha" >> "$GITHUB_ENV"
echo "SOURCE_ROOT=$source_root" >> "$GITHUB_ENV"
echo "CARGO_TARGET_DIR=$target_root" >> "$GITHUB_ENV"
echo "VIZE_PROP_ACCURACY_ARTIFACTS=$RUNNER_TEMP/doctor-prop-packets" >> "$GITHUB_ENV"
mkdir -p "$RUNNER_TEMP/doctor-prop-packets"
printf '%s\n' "$source_sha" > "$RUNNER_TEMP/doctor-prop-packets/source-sha.txt"
git -C "$source_root" status --porcelain > "$RUNNER_TEMP/doctor-prop-packets/source-status.txt"
