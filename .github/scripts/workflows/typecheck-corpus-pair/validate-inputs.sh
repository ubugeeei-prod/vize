#!/usr/bin/env bash
set -euo pipefail
[[ "$BASE_SHA" =~ ^[0-9a-f]{40}$ && "$HEAD_SHA" =~ ^[0-9a-f]{40}$ ]]
# One SHA twice is an A/A run of the corpus pair; owned pairs need two builds.
[[ "$OWNED_PAIR" != true || "$BASE_SHA" != "$HEAD_SHA" ]]
[[ "$RUNS" =~ ^[1-9][0-9]?$ && "$CORPORA" =~ ^[a-z0-9,-]*$ ]]
