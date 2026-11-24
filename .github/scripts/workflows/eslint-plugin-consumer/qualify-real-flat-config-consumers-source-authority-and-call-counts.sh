#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/eslint-plugin-consumer/qualify-real-flat-config-consumers-source-authority-and-call-counts.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
node --test npm/linter/eslint/test/consumer.test.mjs 2>&1 | tee "$VIZE_ESLINT_EVIDENCE/consumer.log"
(cd npm/linter/oxlint && vp test run src/file-state.test.ts src/batched-rules.test.ts src/active-rule-collection.test.ts) 2>&1 | tee "$VIZE_ESLINT_EVIDENCE/shared-routing.log"
node npm/linter/oxlint/src/eslint-acceptance.test.mjs 2>&1 | tee "$VIZE_ESLINT_EVIDENCE/legacy-export.log"
for round in 1 2 3 4 5; do
  node npm/linter/eslint/test/startup-profile.mjs > "$VIZE_ESLINT_EVIDENCE/startup-$round.json"
done
