#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-pr-performance/scripts/package-evidence-without-traversing-dependency-symlinks.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ -d performance-observation ]]; then
  tar --exclude=node_modules --exclude=.vize --exclude=__out__ \
    -czf performance-observation.tgz performance-observation
fi
