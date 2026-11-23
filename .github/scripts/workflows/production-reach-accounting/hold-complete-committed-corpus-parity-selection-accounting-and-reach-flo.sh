#!/usr/bin/env bash
set -o pipefail
cargo test --locked --profile ci -p vize_atelier_sfc --features legacy-dom-differential --test davinci_production_reach -- --nocapture 2>&1 | tee production-reach-corpus.log
