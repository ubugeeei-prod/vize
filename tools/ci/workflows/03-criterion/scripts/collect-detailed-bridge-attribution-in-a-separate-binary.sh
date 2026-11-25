#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-criterion/scripts/collect-detailed-bridge-attribution-in-a-separate-binary.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp exec cargo clippy -p vize_l1_to_l2 --features davinci-benchmark-profile --lib -- -D warnings -D clippy::wildcard_imports
vp exec cargo clippy -p vize_atelier_sfc --features davinci-production-profile --example davinci_production_perf -- -D warnings -D clippy::wildcard_imports
vp exec cargo build --profile ci-opt -p vize_atelier_sfc --features davinci-production-profile --example davinci_production_perf
target/ci-opt/examples/davinci_production_perf production-perf/bridge-attribution.json --attribution-only
