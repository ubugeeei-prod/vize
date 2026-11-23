#!/usr/bin/env bash
set -euo pipefail
default_status=0
VIZE_TEMPLATE_EMIT_TS40_CAPTURE="$EVIDENCE/default" \
  cargo test --locked --profile ci -p vize_maestro --test davinci_ts40_projection -- --nocapture \
  > "$EVIDENCE/default.stdout" 2> "$EVIDENCE/default.stderr" || default_status=$?
legacy_status=0
VIZE_TEMPLATE_EMIT_TS40_CAPTURE="$EVIDENCE/legacy" \
  cargo test --locked --profile ci -p vize_maestro --features legacy --test davinci_ts40_projection -- --nocapture \
  > "$EVIDENCE/legacy.stdout" 2> "$EVIDENCE/legacy.stderr" || legacy_status=$?
printf '{"default":%s,"legacy":%s,"nativeConsumerAcceptance":false}\n' \
  "$default_status" "$legacy_status" > "$EVIDENCE/status.json"
test "$default_status" = 0
test "$legacy_status" = 0
