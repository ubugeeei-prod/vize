#!/usr/bin/env bash
node "$VIZE_CI_FILES/read-the-pinned-toolchain.ts" >> "$GITHUB_OUTPUT"
