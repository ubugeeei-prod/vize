#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-criterion/scripts/measure-shipping-adapter-pairs.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
mkdir -p production-perf
target/ci-opt/examples/davinci_production_perf production-perf/samples.json
{
  git rev-parse HEAD
  rustc --version --verbose
  uname -a
  lscpu
} > production-perf/runner.txt
