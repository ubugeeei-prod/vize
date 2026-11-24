#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/davinci-canon-scaling/bind-the-original-warm-request-pair-to-its-actual-source-baseline.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ "$ORIGINAL_PUBLIC_REQUESTED" == "true" ]]; then
  vp exec node tests/performance/support/warm-type-backed-public.ts prepare
else
  vp exec node tests/performance/support/warm-type-backed-workflow.ts prepare
fi
