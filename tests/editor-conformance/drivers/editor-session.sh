#!/bin/sh
"$@"
status=$?
sleep 5
exit "$status"
