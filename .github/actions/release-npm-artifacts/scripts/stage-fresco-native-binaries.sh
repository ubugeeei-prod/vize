#!/usr/bin/env bash
node tools/support/github/clean-node-binaries.ts npm/fresco-native
node tools/support/github/collect-native-artifacts.ts npm/fresco-native/artifacts npm/fresco-native fresco-native
