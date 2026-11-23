#!/usr/bin/env bash
set -euo pipefail
arguments=()
if [[ -n "$PACKAGE_ROUTE_BASE" ]]; then arguments+=(--route-work-base "$PACKAGE_ROUTE_BASE"); fi
node tools/benchmarks/scripts/workspace-route-identity-admission.ts --source-root "$GITHUB_WORKSPACE" --out "$RUNNER_TEMP/workspace-identity" "${arguments[@]}"
