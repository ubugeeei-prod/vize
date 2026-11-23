#!/usr/bin/env bash
sed -n -f "$VIZE_CI_FILES/toolchain-channel-output.sed" rust-toolchain.toml >> "$GITHUB_OUTPUT"
