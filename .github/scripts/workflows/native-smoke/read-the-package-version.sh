#!/usr/bin/env bash
node "$VIZE_CI_FILES/read-the-package-version.cjs" >> "$GITHUB_OUTPUT"
