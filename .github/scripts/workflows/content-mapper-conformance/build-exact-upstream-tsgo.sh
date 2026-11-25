#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/content-mapper-conformance/build-exact-upstream-tsgo.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
go build -tags=noembed -trimpath -o "$RUNNER_TEMP/tsgo" ./cmd/tsc
cp internal/bundled/libs/*.d.ts "$RUNNER_TEMP/"
