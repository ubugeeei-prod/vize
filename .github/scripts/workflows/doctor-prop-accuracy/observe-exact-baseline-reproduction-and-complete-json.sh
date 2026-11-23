#!/usr/bin/env bash
cd "$SOURCE_ROOT"
status=0
/usr/bin/time -v cargo test --locked --profile ci -p vize --test integration doctor_prop_accuracy_cli::model_alias_and_native_fallthrough_props_preserve_the_complete_healthy_packet -- --nocapture > "$VIZE_PROP_ACCURACY_ARTIFACTS/test.stdout" 2> "$VIZE_PROP_ACCURACY_ARTIFACTS/test.stderr" || status=$?
cat "$VIZE_PROP_ACCURACY_ARTIFACTS/test.stdout"
test "$status" -ne 0
node "$VIZE_CI_FILES/verify-diagnostic-packet.ts"
