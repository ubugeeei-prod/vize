#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/turbopack-workflow/run-selected-producer.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
case "$*" in *turbopack-workflow.test.ts*) exit 0;; esac
printf "intentional producer failure\n"
exit 23
