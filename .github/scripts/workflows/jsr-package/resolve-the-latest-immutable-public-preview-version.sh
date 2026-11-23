#!/usr/bin/env bash
echo "version=$(node tools/support/release/jsr/package.ts latest-public)" >> "$GITHUB_OUTPUT"
