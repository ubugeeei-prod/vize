#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/05-editor-conformance/scripts/install-the-repository-rust-toolchain-for-vs-code-packaging.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
rustup toolchain install || rustup show active-toolchain
cargo --version
