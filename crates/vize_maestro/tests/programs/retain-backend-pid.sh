#!/bin/sh
echo "$$" >> "${0%/*}/backend.pids"
exec cat 3>&1 > /dev/null
