#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/pr-performance-observation/package-evidence-without-traversing-dependency-symlinks.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ -d performance-observation ]]; then
  tar --exclude=node_modules --exclude=.vize --exclude=__out__ \
    -czf performance-observation.tgz performance-observation
fi
