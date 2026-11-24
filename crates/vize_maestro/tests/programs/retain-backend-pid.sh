#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/crates/vize_maestro/tests/programs/retain-backend-pid.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
echo "$$" >> "${0%/*}/backend.pids"
exec cat 3>&1 > /dev/null
