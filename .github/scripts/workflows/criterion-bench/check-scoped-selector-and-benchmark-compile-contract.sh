#!/usr/bin/env bash
vp exec cargo test -p vize_atelier_vapor --features davinci-benchmark --lib compile::benchmark::tests
vp exec cargo clippy -p vize_atelier_sfc --features davinci-production-bench --example davinci_production_perf -- -D warnings -D clippy::wildcard_imports
