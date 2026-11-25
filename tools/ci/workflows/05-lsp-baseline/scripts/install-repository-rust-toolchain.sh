#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/05-lsp-baseline/scripts/install-repository-rust-toolchain.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
rustup toolchain install || rustup show active-toolchain
cargo --version
