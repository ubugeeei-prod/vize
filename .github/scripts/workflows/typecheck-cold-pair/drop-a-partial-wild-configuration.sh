#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-cold-pair/drop-a-partial-wild-configuration.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
sed -i -f "$VIZE_CI_FILES/drop-wild-rustflags.sed" "$HOME/.cargo/config.toml" || true
