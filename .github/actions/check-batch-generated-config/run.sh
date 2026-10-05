#!/usr/bin/env bash
set -euo pipefail
cargo test --locked --profile ci-opt -p vize_canon --lib nested_generated_config:: -- --nocapture
cargo test --locked --profile ci-opt -p vize_canon --test tier_l_incremental failure::tests:: -- --nocapture
cargo test --locked -p vize_l1_to_l2 --features legacy-differential --test davinci_lowering_corpus -- --nocapture
cargo test --locked -p vize_l1_to_l2 --features legacy-differential --test davinci_dom_corpus -- --nocapture
cargo test --locked -p vize_l1_to_l2 --features legacy-differential --test davinci_remarks_corpus -- --nocapture
cargo test --locked -p vize_canon --features legacy-differential --test davinci_projection_differential -- --nocapture
