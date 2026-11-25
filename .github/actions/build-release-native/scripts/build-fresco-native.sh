#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/build-release-native/scripts/build-fresco-native.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
args=(npm/fresco-native ../../crates/vize_fresco/Cargo.toml vize_fresco "$RELEASE_TARGET")
if [[ "$RELEASE_CROSS_COMPILE" == true ]]; then args+=(--cross-compile); fi
node tools/support/github/build-napi-package.ts "${args[@]}"
if [[ "$RUNNER_OS" == Linux && "$RELEASE_TARGET" == *-gnu ]]; then
  node tools/commands/ci/github/verify-glibc-symbols.ts npm/fresco-native/*.linux-*-gnu.node
fi
