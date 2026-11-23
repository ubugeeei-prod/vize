#!/usr/bin/env bash
sed -n 's/^channel = "\(.*\)"$/channel=\1/p' rust-toolchain.toml >> "$GITHUB_OUTPUT"
