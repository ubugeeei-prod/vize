#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/05-editor-conformance/scripts/install-the-repository-rust-toolchain.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
rustup --version
rustup toolchain install || rustup show active-toolchain
cargo --version
