#!/usr/bin/env bash
set -euo pipefail
mkdir -p target/vize-tests/tag-prefix-classification-10622
node tools/support/typescript/check-project.ts 2>&1 | tee target/vize-tests/tag-prefix-classification-10622/typescript.log
