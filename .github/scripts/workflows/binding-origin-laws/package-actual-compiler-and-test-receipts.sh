#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$RUNNER_TEMP/binding-origin-admission"
tar --exclude=cargo-target --exclude=providers --exclude="*.sock" -czf "$RUNNER_TEMP/binding-origin-admission.tgz" -C "$RUNNER_TEMP" binding-origin-admission
mkdir -p "$RUNNER_TEMP/binding-origin-admission/native-metadata"
tar -czf "$RUNNER_TEMP/binding-origin-native-metadata.tgz" -C "$RUNNER_TEMP/binding-origin-admission" native-metadata
