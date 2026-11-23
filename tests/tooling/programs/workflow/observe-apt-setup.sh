#!/usr/bin/env bash
printf '%s' "$$" > "$PROBE_DIR/parent-pid"
. "$1"
