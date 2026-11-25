#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/full-qualification/verify-declarations-build-native-package-and-check-ui-package.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp run --filter './npm/fresco-native' check:generated && vp run --filter './npm/fresco-native' check:types && vp run --filter './npm/native' build:ci && vp run --filter './npm/ui' check
