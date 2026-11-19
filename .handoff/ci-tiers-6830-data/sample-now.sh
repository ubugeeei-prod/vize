#!/bin/bash
# sample-now.sh <pattern> <out.sample>: sample a running process and print test-module frames
pid=$(pgrep -f "$1" | head -1)
echo "pid=$pid"
sample "$pid" 8 -file "$2" >/dev/null 2>&1
grep -oE "[0-9]+ _RNv[^ ]*empty_vue_star_exports[^ ]*" "$2" | awk '{n[$2]+=$1} END{for(k in n) print n[k], k}' | sort -rn | sed -n '1,45p' | sed -E 's/_RNv[A-Za-z0-9_]*22empty_vue_star_exports//' | cut -c1-170
