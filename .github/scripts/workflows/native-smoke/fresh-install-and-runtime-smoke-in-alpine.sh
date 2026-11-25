#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/native-smoke/fresh-install-and-runtime-smoke-in-alpine.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
docker run --rm \
  --env VIZE_SMOKE_TIMING_FILE=/evidence/packed-smoke-timing.jsonl \
  --env "VIZE_SMOKE_EXPECTED_NATIVE_PACKAGE=$NATIVE_PACKAGE" \
  --env "NATIVE_PLATFORM=$NATIVE_PLATFORM" \
  --volume "$GITHUB_WORKSPACE:/work" \
  --volume "$RUNNER_TEMP:/evidence" \
  --volume "$VIZE_CI_FILES/smoke-in-alpine-container.sh:/vize-ci-smoke.sh:ro" \
  --workdir /work \
  node:24-alpine@sha256:a34ab6d7f6f4813f4d0c5469d74685604b962fbdbf5793a7bc7b4998ecb867c8 \
  sh -eux /vize-ci-smoke.sh
