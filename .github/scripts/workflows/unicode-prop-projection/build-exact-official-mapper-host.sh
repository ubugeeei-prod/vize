#!/usr/bin/env bash
test "$(git rev-parse HEAD)" = "$CONTENT_MAPPER_TYPESCRIPT_SHA"
go build -tags=noembed -trimpath -o "$RUNNER_TEMP/tsgo" ./cmd/tsc
cp internal/bundled/libs/*.d.ts "$RUNNER_TEMP/"
