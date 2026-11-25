#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release/cut-release-candidate.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/support/release/release/main.ts cut ${RELEASE_VERSION:+"$RELEASE_VERSION"}
