#!/bin/sh
: "${NATIVE_LAUNCHER_ARGS:?}"
printf '%s\n' "$@" > "$NATIVE_LAUNCHER_ARGS"
