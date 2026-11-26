#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-criterion/scripts/check-scoped-selector-and-benchmark-compile-contract.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp exec cargo test -p vize_atelier_vapor --features davinci-benchmark --lib compile::benchmark::tests
vp exec cargo clippy -p vize_atelier_sfc --features davinci-production-bench --example davinci_production_perf -- -D warnings -D clippy::wildcard_imports
