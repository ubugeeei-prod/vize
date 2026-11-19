#!/bin/bash
# run-sample.sh <repo> <out.json>
D=$(dirname "$0")
node "$D/selection-sample.mjs" "$1" "$2" $(cat "$D/sample-commits.txt")
