#!/bin/bash
printf '%s\n' "$*" >> "$TAR_LOG"
exec /usr/bin/tar "$@"
