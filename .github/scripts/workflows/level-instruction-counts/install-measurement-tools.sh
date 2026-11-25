#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/level-instruction-counts/install-measurement-tools.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
rustup toolchain install 1.100.0 --profile minimal
sudo apt-get update -qq && sudo apt-get install -y --no-install-recommends valgrind
