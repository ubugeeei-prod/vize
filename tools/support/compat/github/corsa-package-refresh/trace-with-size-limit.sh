#!/usr/bin/env bash
ulimit -f 524288
exec "$@"
