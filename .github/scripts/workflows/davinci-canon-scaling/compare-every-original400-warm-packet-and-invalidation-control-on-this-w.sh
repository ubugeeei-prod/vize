#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/davinci-canon-scaling/compare-every-original400-warm-packet-and-invalidation-control-on-this-w.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ "$ORIGINAL_PUBLIC_REQUESTED" == "true" ]]; then
  timeout --signal=TERM --kill-after=30s 20m vp exec node tests/performance/support/warm-type-backed-public.ts measure
else
  timeout --signal=TERM --kill-after=30s 20m vp exec node tests/performance/support/warm-type-backed-requests.ts "$RUNNER_TEMP/warm-before" "$RUNNER_TEMP/warm-pair"
fi
