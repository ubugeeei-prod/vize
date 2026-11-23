#!/usr/bin/env bash
node tools/commands/ci/github/verify-glibc-symbols.ts "$RUNNER_TEMP/floor/vize" "$RUNNER_TEMP/floor/vize-vitrine.node"
# The host recipe is expected to break the floor (DT_RELR, GLIBC_2.39); record it only.
node tools/commands/ci/github/verify-glibc-symbols.ts --max 2.39 "$RUNNER_TEMP/host/vize" "$RUNNER_TEMP/host/vize-vitrine.node" || true
ls -l "$RUNNER_TEMP/host" "$RUNNER_TEMP/floor"
