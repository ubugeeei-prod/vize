#!/usr/bin/env bash
go test ./internal/contentmapper -run '^(TestRunnerTransform|TestRunnerTransformResponseValidation)$' -count=1
go test ./internal/spanmap -run '^(TestOriginalToVirtualOverlappingSpans|TestValidateOriginalOverlapAndFeatures)$' -count=1
