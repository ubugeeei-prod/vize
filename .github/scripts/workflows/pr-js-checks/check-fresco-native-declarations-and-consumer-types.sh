#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/pr-js-checks/check-fresco-native-declarations-and-consumer-types.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp run --filter './npm/fresco-native' check:generated && vp run --filter './npm/fresco-native' check:types
