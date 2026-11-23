#!/usr/bin/env bash
vp run --filter './npm/native' build:ci
node tools/support/github/stage-built-native-packages.ts npm/native
vp run --filter './npm/cli' build
vp run --filter './npm/builder/vite' build
vp run --filter './npm/linter/oxlint' build
