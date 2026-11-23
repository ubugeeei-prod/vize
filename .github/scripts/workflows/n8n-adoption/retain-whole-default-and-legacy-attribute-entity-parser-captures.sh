#!/usr/bin/env bash
qualification="$GITHUB_WORKSPACE/target/differential/attribute-entity-qualification-8142/${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}"
mkdir -p "$qualification"
git rev-parse HEAD > "$qualification/source-sha.txt"
rustc -Vv > "$qualification/rustc.txt"
cargo -V > "$qualification/cargo.txt"
failed=0
for phase in default legacy-native; do
  phase_dir="$qualification/$phase"
  mkdir -p "$phase_dir/tokenizer-captures"
  entity_packets=target/differential/attribute-entity-consumer-8142
  if [[ -d "$entity_packets" ]]; then
    mv "$entity_packets" "$phase_dir/preexisting-attribute-packets"
  fi
  args=(--locked --profile ci -p vize_armature --lib)
  if [[ "$phase" == legacy-native ]]; then
    args+=(--features legacy,native-lex-parity)
  fi
  set +e
  VIZE_TOKENIZER_CAPTURE_DIR="$phase_dir/tokenizer-captures" \
    cargo test "${args[@]}" -- --nocapture 2>&1 | tee "$phase_dir/cargo.log"
  pipeline_status=("${PIPESTATUS[@]}")
  test_status=${pipeline_status[0]}
  capture_status=${pipeline_status[1]}
  set -e
  printf '%s\n' "$test_status" > "$phase_dir/status.txt"
  printf '%s\n' "$capture_status" > "$phase_dir/capture-status.txt"
  if [[ -d "$entity_packets" ]]; then
    mv "$entity_packets" "$phase_dir/attribute-packets"
  fi
  if [[ "$test_status" != 0 || "$capture_status" != 0 ]]; then
    failed=1
  fi
done
exit "$failed"
