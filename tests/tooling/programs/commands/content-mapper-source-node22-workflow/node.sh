#!/bin/bash
if [ "$1" = "npm/native/scripts/native-build-receipt.mjs" ]; then printf "producer receipt\n" > "$MOCK_PRODUCER"; exit 0; fi
mkdir -p "$3"
printf "controlled consumer stderr\n" >&2
printf "%s\n" "$@" > "$MOCK_ARGS"
printf "failed outcome\n" > "$3/receipt.json"
exit 23
