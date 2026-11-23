#!/usr/bin/env bash
node tools/support/release/jsr/package.ts channel >> "$GITHUB_OUTPUT"
