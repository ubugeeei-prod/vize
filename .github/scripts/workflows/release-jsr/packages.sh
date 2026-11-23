#!/usr/bin/env bash
node tools/support/release/jsr/package.ts matrix >> "$GITHUB_OUTPUT"
