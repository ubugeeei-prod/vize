#!/bin/bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/setup-rust-script-prebuilt/report-installed-version.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
printf '%s\n' "$*" >> "$RUNNER_LOG"
printf 'rust-script @@VERSION@@\n'
exit @@STATUS@@
