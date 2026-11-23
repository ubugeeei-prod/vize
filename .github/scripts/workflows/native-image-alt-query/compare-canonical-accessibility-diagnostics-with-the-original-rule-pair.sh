#!/usr/bin/env bash
set -euo pipefail
capture="$RUNNER_TEMP/native-image-alt"
baseline="$RUNNER_TEMP/native-image-alt-base"
base_target="$RUNNER_TEMP/native-image-alt-targets/$BASE_SHA"
head_target="$RUNNER_TEMP/native-image-alt-targets/$HEAD_SHA"
probe=crates/vize_patina/tests/canonical_rule_group
base_probe=crates/vize_patina/tests/ci_canonical_lint_probe
cp -R "$probe" "$baseline/$base_probe"
# Compile the same observer with the two original public rule calls.
# Only the head invokes the grouped entry. Both read the same input bytes.
printf '%s\n' 'pub(super) use super::{regions::original_preorder as preorder, separate as requested};' > "$baseline/$base_probe/group.rs"
cat >> "$baseline/crates/vize_patina/Cargo.toml" <<'TOML'

[[test]]
name = "ci_canonical_lint_probe"
path = "tests/ci_canonical_lint_probe/main.rs"
TOML
export VIZE_CANONICAL_LINT_INPUTS="$RUNNER_TEMP/canonical-lint-inputs"
git ls-files -z -- '*.vue' '*.vue.txt' '*.jsx' '*.tsx' '*.jsx.txt' '*.tsx.txt' |
  while IFS= read -r -d '' path; do
    printf '%s\0' "$GITHUB_WORKSPACE/$path"
  done > "$VIZE_CANONICAL_LINT_INPUTS"
base=(--locked --profile ci-opt -p vize_patina --test ci_canonical_lint_probe)
head=(--locked --profile ci-opt -p vize_patina --features canonical-rule-group)
(cd "$baseline" && CARGO_TARGET_DIR="$base_target" VIZE_CANONICAL_LINT_CAPTURE="$capture/base-rule-packets.txt" cargo test "${base[@]}" original_sources -- --exact --ignored --nocapture)
export CARGO_TARGET_DIR="$head_target"
VIZE_CANONICAL_LINT_CAPTURE="$capture/head-rule-packets.txt" cargo test "${head[@]}" --test canonical_rule_group original_sources -- --exact --ignored --nocapture
cmp "$capture/base-rule-packets.txt" "$capture/head-rule-packets.txt"
cargo test "${head[@]}" --lib exhausted_walk_overrides_each_rule
cargo clippy "${head[@]}" --lib --test canonical_rule_group --no-deps -- -D warnings
cargo test "${head[@]}" --test integration native_canonical_
cargo test "${head[@]}" --test canonical_rule_group
(cd "$baseline" && CARGO_TARGET_DIR="$base_target" cargo test "${base[@]}" profile::individual_lint_requests -- --exact --ignored --nocapture 2>&1 | tee "$capture/base-rule-times.log")
cargo test "${head[@]}" --test canonical_rule_group profile::individual_lint_requests -- --exact --ignored --nocapture 2>&1 | tee "$capture/head-rule-times.log"
(cd "$baseline" && CARGO_TARGET_DIR="$base_target" cargo test "${base[@]}" profile::paired_lint_requests -- --exact --ignored --nocapture 2>&1 | tee "$capture/base-grouped-rule-times.log")
cargo test "${head[@]}" --test canonical_rule_group profile::paired_lint_requests -- --exact --ignored --nocapture 2>&1 | tee "$capture/grouped-rule-times.log"
(cd "$baseline" && CARGO_TARGET_DIR="$base_target" cargo test "${base[@]}" profile::original_request_costs -- --exact --ignored --nocapture 2>&1 | tee "$capture/base-original-grouped-times.log")
cargo test "${head[@]}" --test canonical_rule_group profile::original_request_costs -- --exact --ignored --nocapture 2>&1 | tee "$capture/original-grouped-times.log"
cargo test --locked --profile ci-opt -p vize_l2 --features sparse-region-walk --lib
(cd "$baseline" && CARGO_TARGET_DIR="$base_target" cargo test "${base[@]}" regions::original_region_events -- --exact --ignored --nocapture 2>&1 | tee "$capture/base-region-events.log")
cargo test "${head[@]}" --test canonical_rule_group regions::original_region_events -- --exact --ignored --nocapture 2>&1 | tee "$capture/head-region-events.log"
for epoch in base head; do
  grep -E '^(nodes|events) ' "$capture/$epoch-region-events.log" > "$capture/$epoch-region-packets.txt"
  test -s "$capture/$epoch-region-packets.txt"
done
cmp "$capture/base-region-packets.txt" "$capture/head-region-packets.txt"
# Repeat source-built processes in alternating order on the same runner.
# Cargo's artifact record identifies the executable for the requested target.
base_bin=$(cd "$baseline" && CARGO_TARGET_DIR="$base_target" cargo test "${base[@]}" --no-run --message-format=json | jq -ers --arg name ci_canonical_lint_probe -f "$VIZE_CI_FILES/linked-test-executable.jq")
head_bin=$(cargo test "${head[@]}" --test canonical_rule_group --no-run --message-format=json | jq -ers --arg name canonical_rule_group -f "$VIZE_CI_FILES/linked-test-executable.jq")
mkdir -p "$capture/repeated"
for round in 0 1 2 3 4 5; do
  order=(base head)
  if ((round % 2)); then order=(head base); fi
  for epoch in "${order[@]}"; do
    bin="$head_bin"
    if [[ "$epoch" == base ]]; then bin="$base_bin"; fi
    for query in profile::individual_lint_requests profile::paired_lint_requests profile::original_request_costs regions::original_region_events; do
      "$bin" "$query" --exact --ignored --nocapture > "$capture/repeated/$epoch-$round-${query##*::}.log" 2>&1
    done
  done
done
git -C "$baseline" diff --exit-code -- crates/vize_patina/src Cargo.lock
