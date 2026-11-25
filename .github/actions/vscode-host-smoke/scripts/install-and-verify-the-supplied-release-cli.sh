#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/vscode-host-smoke/scripts/install-and-verify-the-supplied-release-cli.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
test -n "$EXPECTED_VERSION"
test -f "$PREBUILT_SERVER"
mkdir -p target/ci
install -m 0755 "$PREBUILT_SERVER" target/ci/vize
test "$(target/ci/vize --version)" = "vize $EXPECTED_VERSION"
