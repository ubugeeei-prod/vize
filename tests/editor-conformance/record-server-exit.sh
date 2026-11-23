#!/bin/sh
code=0
"$@" || code=$?
printf '{"session":%s,"t":null,"event":"exit","code":%s}\n' "$VIZE_TAP_SESSION" "$code" >> "$VIZE_CONFORMANCE_TRANSCRIPT"
exit "$code"
