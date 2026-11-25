#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/content-mapper-conformance/verify-upstream-content-mapper-protocol-regressions.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
go test ./internal/contentmapper -run '^(TestRunnerTransform|TestRunnerTransformResponseValidation)$' -count=1
go test ./internal/spanmap -run '^(TestOriginalToVirtualOverlappingSpans|TestValidateOriginalOverlapAndFeatures)$' -count=1
