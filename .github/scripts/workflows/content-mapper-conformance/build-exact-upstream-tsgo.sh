#!/usr/bin/env bash
go build -tags=noembed -trimpath -o "$RUNNER_TEMP/tsgo" ./cmd/tsc
cp internal/bundled/libs/*.d.ts "$RUNNER_TEMP/"
