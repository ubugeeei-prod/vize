#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/benchmark/record-benchmark-build-provenance.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
{
  printf 'profile=%s\n' "$VIZE_BENCH_BUILD_PROFILE_KEY"
  rustc --version --verbose
  sha256sum base/target/ci-opt/vize head/target/ci-opt/vize
} > benchmark-provenance.txt
