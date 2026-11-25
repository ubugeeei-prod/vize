#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/davinci-resource-budgets/measure-the-complete-ts-44-campaign-with-the-shipped.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
rust-script tools/commands/davinci/resource-budgets.rs \
  --measure \
  --preset linux-x64-ci \
  --runs 3 \
  --keystrokes 20 \
  --idle-seconds 60 \
  --server target/x86_64-unknown-linux-gnu/release/vize \
  --out "${RUNNER_TEMP}/release-resource.json"
