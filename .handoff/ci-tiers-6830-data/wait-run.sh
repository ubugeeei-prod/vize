#!/bin/bash
# wait-run.sh <run-id>... : print each run's conclusion when it completes
pending=("$@")
while [ ${#pending[@]} -gt 0 ]; do
  next=()
  for id in "${pending[@]}"; do
    s=$(gh api "repos/ubugeeei-prod/vize/actions/runs/$id" --jq '.status+" "+(.conclusion // "")' 2>/dev/null || echo "error")
    case "$s" in
      completed*) echo "run $id $s" ;;
      *) next+=("$id") ;;
    esac
  done
  pending=("${next[@]}")
  [ ${#pending[@]} -gt 0 ] && sleep 60
done
