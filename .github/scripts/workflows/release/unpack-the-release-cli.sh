#!/usr/bin/env bash
mkdir -p .artifacts/cli
tar -xzf ".artifacts/cli-archive/vize-$TARGET.tar.gz" -C .artifacts/cli
node tools/commands/ci/github/verify-glibc-symbols.ts .artifacts/cli/vize \
  "npm/native/npm/$PLATFORM/vize-vitrine.$PLATFORM.node"
