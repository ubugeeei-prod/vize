#!/bin/sh
if [ "$1" = "--version" ]; then printf "vize 1.0.0-dev\n"; exit 0; fi
cat
printf "observer stderr 🧪\n" >&2
exit 23
