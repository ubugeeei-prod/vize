#!/usr/bin/env bash
node --test tools/benchmarks/scripts/content-mapper-span-profile.test.ts
node tools/benchmarks/scripts/content-mapper-span-profile.ts build "$RUNNER_TEMP/span-evidence"
