#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/release-npm-artifacts/scripts/lay-out-native-platform-packages.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp exec napi create-npm-dirs
vp exec napi artifacts --output-dir artifacts
vp exec napi pre-publish -t npm --no-gh-release --skip-optional-publish
