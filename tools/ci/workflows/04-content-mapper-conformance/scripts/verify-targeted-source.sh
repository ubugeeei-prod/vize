#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/04-content-mapper-conformance/scripts/verify-targeted-source.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test "$GITHUB_SHA" = "$EXPECTED_SOURCE_SHA"
