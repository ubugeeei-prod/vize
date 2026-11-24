#!/usr/bin/env bash
echo "selected=$SELECTED cli=$CLI workers=$WORKERS"
[[ "$CLI" == success ]]
if [[ "$SELECTED" == true ]]; then [[ "$WORKERS" == success ]]
else [[ "$SELECTED" == false && "$WORKERS" == skipped ]]; fi
