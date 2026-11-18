#!/bin/bash
# usage: watch-runs.sh <run-id>...  emits failed jobs as they land and each run's final conclusion; exits when all done
TOKEN=$(gh auth token)
declare -A seen done_
while true; do
  all=1
  for id in "$@"; do
    [ -n "${done_[$id]}" ] && continue
    all=0
    run=$(curl -s -H "Authorization: Bearer $TOKEN" "https://api.github.com/repos/ubugeeei-prod/vize/actions/runs/$id" || true)
    st=$(echo "$run" | jq -r '.status // "?"'); co=$(echo "$run" | jq -r '.conclusion // ""'); name=$(echo "$run" | jq -r '.name // "?"')
    jobs=$(curl -s -H "Authorization: Bearer $TOKEN" "https://api.github.com/repos/ubugeeei-prod/vize/actions/runs/$id/jobs?per_page=100" || true)
    while IFS= read -r line; do
      [ -z "$line" ] && continue
      if [ -z "${seen[$id:$line]}" ]; then seen[$id:$line]=1; echo "[$name $id] job $line"; fi
    done < <(echo "$jobs" | jq -r '.jobs[]? | select(.conclusion=="failure" or .conclusion=="cancelled" or .conclusion=="timed_out") | "\(.name): \(.conclusion)"')
    if [ "$st" = "completed" ]; then echo "[$name $id] RUN DONE: $co"; done_[$id]=1; fi
  done
  [ $all = 1 ] && exit 0
  sleep 60
done
