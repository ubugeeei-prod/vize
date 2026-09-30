#!/usr/bin/env bash
# Compare the checked-out PR merge against its actual base parent.
set -euo pipefail

BASELINE_REV="${BASELINE_REV:-}"
case "$BASELINE_REV" in 0000000000000000000000000000000000000000) BASELINE_REV="";; esac
# pull_request checkout is a merge commit. The event's base SHA can
# predate commits that landed on the base before this job started.
if git rev-parse --verify -q HEAD^2 >/dev/null; then
  BASELINE_REV="$(git rev-parse HEAD^1)"
fi
SEMVER_CHANGE_MARKER="$(cat "$RUNNER_TEMP/semver-change-marker.txt")"
SEMVER_ARGS=()
if printf '%s\n' "$SEMVER_CHANGE_MARKER" | grep -Eq '^[[:alnum:]_-]+(\([^)]+\))?!:|^BREAKING CHANGE:'; then
  SEMVER_ARGS+=(--release-type major)
fi
semver_baseline_dir="$(mktemp -d "$RUNNER_TEMP/semver-baseline.XXXXXX")"
semver_output="$(mktemp "$RUNNER_TEMP/semver-output.XXXXXX")"
trap 'rm -rf "$semver_baseline_dir"; rm -f "$semver_output"' EXIT
if [ -n "$BASELINE_REV" ]; then
  semver_baseline_root="$(rust-script tools/commands/ci/github/semver-baseline.rs "$1" "$semver_baseline_dir" "$BASELINE_REV")"
else
  semver_baseline_root="$(rust-script tools/commands/ci/github/semver-baseline.rs "$1" "$semver_baseline_dir")"
fi
if [ -n "$semver_baseline_root" ]; then
  SEMVER_ARGS+=(--baseline-root "$semver_baseline_root")
elif [ -n "$BASELINE_REV" ]; then
  SEMVER_ARGS+=(--baseline-rev "$BASELINE_REV")
fi

semver_status=0
cargo semver-checks check-release --package "$1" --color never "${SEMVER_ARGS[@]}" > "$semver_output" 2>&1 || semver_status=$?
cat "$semver_output"
if [ "$semver_status" -ne 0 ]; then
  if [ "$semver_status" -ne 1 ]; then
    exit "$semver_status"
  fi
  if [ "$1" != vize_armature ]; then
    exit "$semver_status"
  fi
  python3 tools/commands/ci/github/allow-armature-tokenizer-reexports.py "$semver_output"
fi

if [ "$1" = vize_armature ]; then
  # The same source must execute as a separate crate against the published
  # 0.429.1 API and this candidate. This covers the re-exports rustdoc cannot
  # resolve for cargo-semver-checks (upstream issue #355). Share target objects
  # between both manifests and omit debug info to bound release-runner disk use.
  (
    export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$RUNNER_TEMP/armature-tokenizer-consumer-target}"
    export CARGO_PROFILE_DEV_DEBUG=0
    export CARGO_PROFILE_TEST_DEBUG=0
    cargo test --lib --locked --manifest-path tests/external-consumers/armature-tokenizer/baseline/Cargo.toml
    # This path dependency changes version with the release branch, so resolve
    # a fresh fixture lock rather than pinning it to the source version.
    cargo test --lib --manifest-path tests/external-consumers/armature-tokenizer/Cargo.toml
  )
fi
