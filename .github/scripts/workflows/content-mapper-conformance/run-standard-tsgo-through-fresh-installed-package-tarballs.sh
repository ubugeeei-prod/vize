#!/usr/bin/env bash
node tools/commands/release/npm/smoke-release-install.ts --prepare-manifests --content-mapper-checks \
  npm/native npm/native/npm/* \
  npm/cli
