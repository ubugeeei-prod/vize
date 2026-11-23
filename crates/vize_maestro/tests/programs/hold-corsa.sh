#!/bin/sh
echo $$ >> backend.pids
exec cat 3>&1 > /dev/null
