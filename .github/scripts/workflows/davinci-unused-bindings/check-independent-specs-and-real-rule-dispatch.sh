#!/usr/bin/env bash
set -o pipefail
cargo test --profile ci --message-format=json-render-diagnostics -p vize_l0 --test fact_demand | tee target/unused-bindings-units/ci-fact_demand.jsonl
cargo test --profile ci --message-format=json-render-diagnostics -p vize_croquis --test fact_spec | tee target/unused-bindings-units/ci-fact_spec.jsonl
cargo test --profile ci --message-format=json-render-diagnostics -p vize_patina unused_setup_bindings | tee target/unused-bindings-units/ci-unused_setup_bindings.jsonl
