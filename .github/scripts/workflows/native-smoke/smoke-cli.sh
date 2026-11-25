#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/native-smoke/smoke-cli.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [ -x target/ci/vize.exe ]; then
  target/ci/vize.exe --version
else
  target/ci/vize --version
fi
