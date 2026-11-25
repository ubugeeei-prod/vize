#!/usr/bin/env bash
rust-script tools/commands/davinci/resource-budgets.rs \
  --measure \
  --preset linux-x64-ci \
  --runs 3 \
  --keystrokes 20 \
  --idle-seconds 60 \
  --server target/x86_64-unknown-linux-gnu/release/vize \
  --out "${RUNNER_TEMP}/release-resource.json"
