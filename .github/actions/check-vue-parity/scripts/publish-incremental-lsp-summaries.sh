#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/check-vue-parity/scripts/publish-incremental-lsp-summaries.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
for suite in check-fixtures-topology check-fixtures-cycles vben-batch-incremental misskey-lsp-incremental vben-lsp-incremental misskey-lsp-churn; do
  if [ -s "target/vize-tests/metrics/$suite/summary.md" ]; then
    cat "target/vize-tests/metrics/$suite/summary.md" >> "$GITHUB_STEP_SUMMARY"
  fi
done
