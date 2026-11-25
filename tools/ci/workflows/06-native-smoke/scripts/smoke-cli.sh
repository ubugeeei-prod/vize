#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/06-native-smoke/scripts/smoke-cli.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [ -x target/ci/vize.exe ]; then
  target/ci/vize.exe --version
else
  target/ci/vize --version
fi
