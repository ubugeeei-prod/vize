#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/content-mapper-replay/validate-inputs-2.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
[[ "$BASE" =~ ^[0-9a-f]{40}$ && "$HEAD" =~ ^[0-9a-f]{40}$ && "$HOST_RUNS" =~ ^[0-9][0-9]?$ ]]
[[ "$CORPORA" =~ ^[a-z0-9-]+(,[a-z0-9-]+)*$ && "$RUNS" =~ ^[0-9][0-9]?$ ]]
