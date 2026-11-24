#!/usr/bin/env bash
mkdir -p .artifacts/cli
tar -xzf ".artifacts/cli-archive/vize-$TARGET.tar.gz" -C .artifacts/cli
node tools/ci/commands/github/verify-glibc-symbols.ts .artifacts/cli/vize
