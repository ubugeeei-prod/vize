#!/usr/bin/env bash
rustup toolchain install || rustup show active-toolchain
cargo --version
