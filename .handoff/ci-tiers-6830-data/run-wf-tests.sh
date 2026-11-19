#!/bin/bash
# run-wf-tests.sh <repo> <list> <log>: run listed node tests, summarize
cd "$1" || exit 1
node --test --test-concurrency=4 $(cat "$2") > "$3" 2>&1
grep -E "^ℹ (tests|pass|fail|skipped)" "$3"
grep -A12 "^✖ failing" "$3" | head -40
