#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$VIZE_TEMPLATE_GLOBAL_CAPTURE_DIR"
git rev-parse HEAD >"$VIZE_TEMPLATE_GLOBAL_CAPTURE_DIR/producer.sha"
if [[ -z "$VIZE_TEMPLATE_GLOBAL_BEFORE" ]]; then unset VIZE_TEMPLATE_GLOBAL_BEFORE; fi
set +e
cargo test --locked --profile ci -p vize_atelier_sfc --test template_global_policy every_sfc_backend_keeps_host_globals_on_the_component_proxy -- --exact --nocapture >"$VIZE_TEMPLATE_GLOBAL_CAPTURE_DIR/test.stdout" 2>"$VIZE_TEMPLATE_GLOBAL_CAPTURE_DIR/test.stderr"
status=$?
set -e
printf '%s\n' "$status" >"$VIZE_TEMPLATE_GLOBAL_CAPTURE_DIR/test.status"
cat "$VIZE_TEMPLATE_GLOBAL_CAPTURE_DIR/test.stdout"
cat "$VIZE_TEMPLATE_GLOBAL_CAPTURE_DIR/test.stderr" >&2
test "$status" = 0
node "$VIZE_CI_FILES/verify-template-global-output.ts" "$VIZE_TEMPLATE_GLOBAL_CAPTURE_DIR"
