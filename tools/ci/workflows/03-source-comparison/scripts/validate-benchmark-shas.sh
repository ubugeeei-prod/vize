#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-source-comparison/scripts/validate-benchmark-shas.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
full_sha='^[0-9a-f]{40}$'
if [[ ! "$BENCHMARK_BASE_SHA" =~ $full_sha ]]; then
  echo "::error title=Invalid base_sha::base_sha must be a full lowercase 40-character commit SHA"
  exit 1
fi
if [[ ! "$BENCHMARK_HEAD_SHA" =~ $full_sha ]]; then
  echo "::error title=Invalid head_sha::head_sha must be a full lowercase 40-character commit SHA"
  exit 1
fi
if [[ "$BENCHMARK_BASE_SHA" == "$BENCHMARK_HEAD_SHA" ]]; then
  echo "::error title=Invalid benchmark range::base_sha must differ from head_sha"
  exit 1
fi
if [[ "$RUN_HEAD_SHA" != "$BENCHMARK_HEAD_SHA" ]]; then
  echo "::error title=Workflow ref mismatch::run head_sha must match $BENCHMARK_HEAD_SHA (got $RUN_HEAD_SHA)"
  exit 1
fi
