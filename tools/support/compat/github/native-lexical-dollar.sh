#!/usr/bin/env bash
# Reuse the Canon ci-opt library compiled immediately before this step.
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
export VIZE_TEST_REQUIRE_TSGO=1 GIT_NO_REPLACE_OBJECTS=1
cd "$NATIVE_PHASE_SOURCE_ROOT"
packet_root="$RUNNER_TEMP/native-source-recipe-lexical-dollar"
[[ ! -e "$packet_root" ]]
mkdir -p "$packet_root"
git --no-replace-objects rev-parse HEAD > "$packet_root/source.sha"
cp -R tests/_fixtures/differential/typechecker/lexical-dollar-slot-6879 "$packet_root/corpus"
cargo_args=(test --locked --profile ci-opt -p vize_canon --lib --no-run --message-format=json
  --config 'profile.ci-opt.inherits="release"'
  --config 'profile.ci-opt.lto="thin"'
  --config 'profile.ci-opt.codegen-units=16')
printf '%s\0' cargo "${cargo_args[@]}" > "$packet_root/cargo.argv.nul"
set +e
cargo "${cargo_args[@]}" > "$packet_root/cargo.stdout" 2> "$packet_root/cargo.stderr"
actual_status=$?
set -e
printf '%s\n' "$actual_status" > "$packet_root/cargo.status"
if [[ "$actual_status" != 0 ]]; then cat "$packet_root/cargo.stdout" "$packet_root/cargo.stderr"; exit "$actual_status"; fi
node tools/support/compat/github/native-lexical-dollar-metadata.mjs "$packet_root"
test_binary="$(node -p 'JSON.parse(require("node:fs").readFileSync(process.argv[1],"utf8")).testBinary.path' "$packet_root/metadata.json")"
phase_status=0
for phase in producer native; do
  case "$phase" in
    producer) selector=virtual_ts::scope::globals::instance::tests::; exact_args=() ;;
    native) selector=batch::type_checker::tests::recent_issues::lexical_dollar_slots::lexical_dollar_slots_preserve_complete_native_diagnostics_and_repairs; exact_args=(--exact) ;;
  esac
  export VIZE_LEXICAL_DOLLAR_CAPTURE_DIR="$packet_root/$phase"
  mkdir -p "$VIZE_LEXICAL_DOLLAR_CAPTURE_DIR"
  printf '%s\0' "$test_binary" "$selector" "${exact_args[@]}" --nocapture --test-threads=1 > "$packet_root/$phase.argv.nul"
  set +e
  "$test_binary" "$selector" "${exact_args[@]}" --nocapture --test-threads=1 > "$packet_root/$phase.stdout" 2> "$packet_root/$phase.stderr"
  actual_status=$?
  set -e
  printf '%s\n' "$actual_status" > "$packet_root/$phase.status"
  cat "$packet_root/$phase.stdout" "$packet_root/$phase.stderr"
  if [[ "$actual_status" != 0 ]]; then phase_status=1; fi
done
unset VIZE_LEXICAL_DOLLAR_CAPTURE_DIR
node tools/support/compat/github/native-lexical-dollar-receipt.mjs "$packet_root"
exit "$phase_status"
