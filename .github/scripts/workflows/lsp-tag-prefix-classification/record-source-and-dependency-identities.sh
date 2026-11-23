#!/usr/bin/env bash
mkdir -p target/vize-tests/tag-prefix-classification-10622
node "$VIZE_CI_FILES/record-source-and-dependency-identities.ts"
