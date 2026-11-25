#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/release-npm-artifacts/scripts/stage-fresco-native-binaries.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/support/github/clean-node-binaries.ts npm/fresco-native
node tools/support/github/collect-native-artifacts.ts npm/fresco-native/artifacts npm/fresco-native fresco-native
