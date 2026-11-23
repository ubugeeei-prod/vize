#!/usr/bin/env bash
sed -i '/# Added by wild GitHub Action/,/^rustflags/d' "$HOME/.cargo/config.toml" || true
