#!/usr/bin/env bash
mkdir -p "$RUNNER_TEMP/typescript-plugin"
git rev-parse HEAD > "$RUNNER_TEMP/typescript-plugin/attempted-source.txt"
