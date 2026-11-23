#!/bin/sh
exec 0<&-
echo 'authoritative compiler failure' >&2
exit 23
