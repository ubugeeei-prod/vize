#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/published-fresh-project/initialize-and-exercise-the-public-npm-release-in-alpine.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
mkdir -p "$RUNNER_TEMP/published-fresh-project"
docker run --rm \
  --volume "$GITHUB_WORKSPACE:/work:ro" \
  --volume "$RUNNER_TEMP/published-fresh-project:/evidence" \
  --workdir /work \
  --env VIZE_PUBLISHED_FRESH_EVIDENCE=/evidence/receipt.json \
  node:24-alpine@sha256:a34ab6d7f6f4813f4d0c5469d74685604b962fbdbf5793a7bc7b4998ecb867c8 \
  node tools/support/release/verify_published/fresh_project.ts "$RELEASE_VERSION"
