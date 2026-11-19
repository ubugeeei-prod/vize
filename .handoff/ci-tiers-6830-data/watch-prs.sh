#!/bin/bash
# watch-prs.sh <pr>... : print state changes until all are merged or closed
prev=""
while true; do
  cur=""
  done_all=1
  for p in "$@"; do
    s=$(gh pr view "$p" --repo ubugeeei-prod/vize --json state --jq .state 2>/dev/null || echo ERR)
    f=$(gh pr checks "$p" --repo ubugeeei-prod/vize 2>/dev/null | grep -c $'\tfail\t')
    cur="$cur ${p}=${s},fail${f}"
    [ "$s" = OPEN ] && done_all=0
  done
  [ "$cur" != "$prev" ] && echo "$cur"
  prev="$cur"
  [ "$done_all" = 1 ] && break
  sleep 60
done
