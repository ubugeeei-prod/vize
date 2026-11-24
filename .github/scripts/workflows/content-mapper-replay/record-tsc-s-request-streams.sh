#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/content-mapper-replay/record-tsc-s-request-streams.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
node tools/benchmarks/scripts/content-mapper-replay-record.ts \
  --bin replay-bin/cli/base/vize --tsc tools/benchmarks/typecheck-hosts/node_modules/.bin/tsc \
  --corpora "$CORPORA" --work "$RUNNER_TEMP/replay" > "$RUNNER_TEMP/streams.args"
cat "$RUNNER_TEMP/streams.args"
