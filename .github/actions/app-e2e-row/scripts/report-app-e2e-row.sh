#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/app-e2e-row/scripts/report-app-e2e-row.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
echo "App E2E row $ROW_ID: $ROW_OUTCOME" >> "$GITHUB_STEP_SUMMARY"
if [[ "$ROW_OUTCOME" != "success" ]]; then
  echo "::error title=App E2E row failed::$ROW_ID failed; inspect its unique artifact"
  exit 1
fi
