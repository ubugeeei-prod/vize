#!/usr/bin/env bash
packets="$RUNNER_TEMP/provide-assertion-packets"
cargo test --locked --profile ci -p vize_croquis --lib typed_assertion_value_keeps_exact_provider_source_and_metadata -- --nocapture > "$packets/producer-unit.stdout" 2> "$packets/producer-unit.stderr"
cargo test --locked --profile ci -p vize_croquis_cf --lib provided_value_wrappers_preserve_runtime_kinds_and_plain_negatives -- --nocapture > "$packets/classifier-unit.stdout" 2> "$packets/classifier-unit.stderr"
node "$VIZE_CI_FILES/verify-producer-units.ts"
