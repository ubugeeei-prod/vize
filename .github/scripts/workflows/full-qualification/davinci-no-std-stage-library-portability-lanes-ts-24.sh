#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/full-qualification/davinci-no-std-stage-library-portability-lanes-ts-24.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo build -p vize_l0 -p vize_l1 -p vize_l2 -p vize_l3 -p vize_l4 -p vize_l1_to_l2 -p vize_l2_to_l3 --lib --target wasm32-wasip2 && cargo build -p vize_l0 -p vize_l1 -p vize_l2 -p vize_l3 -p vize_l4 -p vize_l1_to_l2 -p vize_l2_to_l3 --lib --target wasm32-wasip2 --no-default-features
