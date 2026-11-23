#!/usr/bin/env bash
rustup --version
rustup toolchain install || rustup show active-toolchain
cargo --version
