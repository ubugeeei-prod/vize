#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-pr-rust/scripts/select-clippy-and-doctest-runtime-envelope.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
case "$GITHUB_EVENT_NAME" in
  pull_request) printf 'VIZE_TEST_DISABLE_TSGO=1\n' >> "$GITHUB_ENV" ;;
  merge_group) printf 'VIZE_TEST_REQUIRE_TSGO=1\n' >> "$GITHUB_ENV" ;;
  *) exit 1 ;;
esac
