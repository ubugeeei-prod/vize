#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/cold-typechecker-scale/package-evidence-without-traversing-workspace-links.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
mkdir -p "$RUNNER_TEMP/cold-scale"
tar --exclude='*/project' --exclude='*/xdg-cache' --exclude='*/tmp' -czf "$RUNNER_TEMP/cold-scale-evidence.tgz" -C "$RUNNER_TEMP" cold-scale
