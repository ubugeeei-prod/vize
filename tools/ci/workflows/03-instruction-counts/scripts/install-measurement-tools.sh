#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-instruction-counts/scripts/install-measurement-tools.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
rustup toolchain install 1.100.0 --profile minimal
sudo apt-get update -qq && sudo apt-get install -y --no-install-recommends valgrind
