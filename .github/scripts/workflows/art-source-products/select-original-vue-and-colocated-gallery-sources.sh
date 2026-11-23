#!/usr/bin/env bash
git ls-files -z -- '*.vue' '*.vue.txt' > "$RUNNER_TEMP/gallery-sources"
