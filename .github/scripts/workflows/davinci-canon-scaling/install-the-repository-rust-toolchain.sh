#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/davinci-canon-scaling/install-the-repository-rust-toolchain.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
rustup toolchain install || rustup show active-toolchain
cargo --version
