#!/bin/bash
printf "%s\n" "$@" >> "$MOCK_PACK"
if [ "$#" -ne 1 ] || [ "$1" != "pack" ]; then exit 91; fi
printf "controlled pack stdout\n"
printf "controlled pack stderr\n" >&2
exit "$PACK_STATUS"
