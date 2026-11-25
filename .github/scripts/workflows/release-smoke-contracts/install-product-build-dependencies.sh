#!/usr/bin/env bash
vp install \
  --frozen-lockfile \
  --prefer-offline \
  --filter . \
  --filter '@vize/tools' \
  --filter './npm/native...' \
  --filter './npm/cli...' \
  --filter './npm/builder/vite...' \
  --filter './npm/linter/oxlint...'
