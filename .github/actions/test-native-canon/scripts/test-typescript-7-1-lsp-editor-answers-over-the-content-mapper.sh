#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-typescript-7-1-lsp-editor-answers-over-the-content-mapper.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
test "$("$VIZE_TEST_CONTENT_MAPPER_TSGO" --version)" = "Version 7.1.0-beta"
cargo test --locked --profile ci -p vize --test integration -- content_mapper_tsgo_editor_accuracy:: --nocapture 2>&1 | tee "$RUNNER_TEMP/editor-accuracy.log"
grep -Eq '^test result: ok\. [1-9][0-9]* passed; 0 failed' "$RUNNER_TEMP/editor-accuracy.log"
