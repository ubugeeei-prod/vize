#!/usr/bin/env bash
sed -i -f "$VIZE_CI_FILES/drop-wild-rustflags.sed" "$HOME/.cargo/config.toml" || true
