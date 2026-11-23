#!/usr/bin/env bash
command -v gdb
command -v cc
mkdir -p "$RUNNER_TEMP/native-ops-pr-private"
gdb --version > "$RUNNER_TEMP/native-ops-pr-private/gdb-version.txt"
cc --version > "$RUNNER_TEMP/native-ops-pr-private/cc-version.txt"
dpkg-query --show gdb libc6 > "$RUNNER_TEMP/native-ops-pr-private/package-versions.txt"
