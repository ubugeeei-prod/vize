#!/usr/bin/env bash
set -euo pipefail
capture="$RUNNER_TEMP/native-image-alt"
checkout="$RUNNER_TEMP/original-region-client"
export CARGO_TARGET_DIR="$RUNNER_TEMP/native-image-alt-targets/$HEAD_SHA"
git worktree add --detach "$checkout" "$BASE_SHA"
for epoch in base head; do
  if [[ "$epoch" == head ]]; then git -C "$checkout" checkout --detach "$HEAD_SHA"; fi
  (
    cd "$checkout"
    probe=crates/vize_patina/tests/ci_original_region_client
    mkdir "$probe"
    cp "$GITHUB_WORKSPACE/crates/vize_patina/tests/canonical_rule_group/event_client.rs" "$probe/main.rs"
    cp "$GITHUB_WORKSPACE/crates/vize_patina/tests/canonical_rule_group/regions.rs" "$probe/regions.rs"
    # Older source trees receive only the empty feature and observer target declarations.
    if ! grep -q '^sparse-region-walk =' davinci/vize_l2/Cargo.toml; then
      sed -i '/^\[features\]$/a sparse-region-walk = []' davinci/vize_l2/Cargo.toml
    fi
    if ! grep -q '^canonical-rule-group =' crates/vize_patina/Cargo.toml; then
      sed -i '/^\[features\]$/a canonical-rule-group = ["vize_l2/sparse-region-walk"]' crates/vize_patina/Cargo.toml
    fi
    cat >> crates/vize_patina/Cargo.toml <<'TOML'

[[test]]
name = "ci_original_region_client"
path = "tests/ci_original_region_client/main.rs"
TOML
    cargo test --locked --profile ci-opt -p vize_patina --features canonical-rule-group --test ci_original_region_client --no-run --message-format=json > "$RUNNER_TEMP/$epoch-region-build.jsonl"
    binary=$(jq -ers '[.[] | select(.reason == "compiler-artifact" and .target.name == "ci_original_region_client")] | last | .executable' "$RUNNER_TEMP/$epoch-region-build.jsonl")
    cp "$binary" "$capture/$epoch-original-region-client"
    readelf -SW "$binary" > "$capture/$epoch-original-region-sections.txt"
    nm -nC "$binary" > "$capture/$epoch-original-region-symbols.txt"
    objcopy --dump-section ".text=$capture/$epoch-original-region-text.bin" "$binary" "$RUNNER_TEMP/$epoch-region-sections.elf"
    sha256sum "$binary" > "$capture/$epoch-original-region-client.sha256"
    "$binary" regions::original_region_events --exact --ignored --nocapture > "$capture/$epoch-original-region-client.log" 2>&1
    git restore -- crates/vize_patina/Cargo.toml davinci/vize_l2/Cargo.toml
    rm "$probe/main.rs" "$probe/regions.rs"
    rmdir "$probe"
  )
done
for round in 0 1 2 3 4 5; do
  order=(base head)
  if ((round % 2)); then order=(head base); fi
  for epoch in "${order[@]}"; do
    "$capture/$epoch-original-region-client" regions::original_region_events --exact --ignored --nocapture > "$capture/repeated/$epoch-$round-standalone-events.log" 2>&1
  done
done
sha256sum "$capture/"*-original-region-text.bin > "$capture/original-region-text.sha256"
cmp "$capture/base-original-region-text.bin" "$capture/head-original-region-text.bin"
