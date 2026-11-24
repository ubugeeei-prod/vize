#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/vue-benchmarks-current-qualification/require-complete-original-lint-and-typecheck-qualification.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
echo "lint=$LINT_OUTCOME typecheck=$TYPECHECK_OUTCOME"
test "$LINT_OUTCOME" = success
test "$TYPECHECK_OUTCOME" = success
