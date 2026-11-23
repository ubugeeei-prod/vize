#!/usr/bin/env bash
if [[ -f "$CARGO_TARGET_DIR/ci/vize" ]]; then
  sha256sum "$CARGO_TARGET_DIR/ci/vize" > "$VIZE_SSR_CONTEXT_ARTIFACTS/binary-sha256.txt"
  "$CARGO_TARGET_DIR/ci/vize" --version > "$VIZE_SSR_CONTEXT_ARTIFACTS/binary-version.txt"
fi
