#!/usr/bin/env bash
if [[ "$ORIGINAL_PUBLIC_REQUESTED" == "true" ]]; then
  vp exec node tests/performance/support/warm-type-backed-public.ts prepare
else
  vp exec node tests/performance/support/warm-type-backed-workflow.ts prepare
fi
