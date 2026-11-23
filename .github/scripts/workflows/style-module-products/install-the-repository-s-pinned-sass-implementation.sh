#!/usr/bin/env bash
version=$(node "$VIZE_CI_FILES/read-pinned-sass-version.ts")
npm install --prefix "$RUNNER_TEMP/scss-oracle" --no-audit --no-fund --ignore-scripts --save-exact "sass@$version"
