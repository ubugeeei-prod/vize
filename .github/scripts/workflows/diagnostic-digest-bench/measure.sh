#!/usr/bin/env bash
set -euo pipefail
chmod +x pair-bin/*/vize
corpora="$RUNNER_TEMP/corpora"
node tools/benchmarks/scripts/diagnostic-digest-bench.ts \
  --bin base=pair-bin/base/vize --bin head=pair-bin/head/vize --bin base-aa=pair-bin/base-aa/vize \
  --elk "$corpora/elk-$ELK_SHA" --misskey "$corpora/misskey-$MISSKEY_SHA/packages/frontend" \
  --rounds "$ROUNDS" --out "$RUNNER_TEMP/digest-bench" > /dev/null
