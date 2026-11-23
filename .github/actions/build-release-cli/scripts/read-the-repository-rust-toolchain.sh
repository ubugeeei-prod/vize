#!/usr/bin/env bash
node "$GITHUB_ACTION_PATH/scripts/read-the-repository-rust-toolchain.ts" >> "$GITHUB_OUTPUT"
