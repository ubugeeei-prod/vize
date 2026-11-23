#!/usr/bin/env bash
echo "lint=$LINT_OUTCOME typecheck=$TYPECHECK_OUTCOME"
test "$LINT_OUTCOME" = success
test "$TYPECHECK_OUTCOME" = success
