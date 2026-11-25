#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-glibc-floor/unpack-the-release-cli.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
mkdir -p .artifacts/cli
tar -xzf ".artifacts/cli-archive/vize-$TARGET.tar.gz" -C .artifacts/cli
node tools/commands/ci/github/verify-glibc-symbols.ts .artifacts/cli/vize
