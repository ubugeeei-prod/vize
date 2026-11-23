#!/usr/bin/env bash
command -v gdb
command -v cc
gdb --version > "$RUNNER_TEMP/native-ops-private/gdb-version.txt"
cc --version > "$RUNNER_TEMP/native-ops-private/control-cc-version.txt"
