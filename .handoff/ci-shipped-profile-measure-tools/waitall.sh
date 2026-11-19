#!/bin/bash
# usage: waitall.sh [file:<path>] [run:<id>] ...  emits one line per finished item, exits when all are done
declare -A done
n=$#
while [ ${#done[@]} -lt $n ]; do
  for it in "$@"; do
    [ -n "${done[$it]:-}" ] && continue
    case $it in
      file:*) f=${it#file:}; if grep -q 'test result\|error\|FAILED' "$f" 2>/dev/null; then done[$it]=1; echo "finished $it"; fi ;;
      run:*) id=${it#run:}; st=$(gh run view "$id" --json status,conclusion --jq '.status+" "+(.conclusion//"")' 2>/dev/null); case $st in completed*) done[$it]=1; echo "run $id $st";; esac ;;
      task:*) f=${it#task:}; if grep -q 'exited with code' "$f" 2>/dev/null; then done[$it]=1; echo "finished $it"; fi ;;
    esac
  done
  sleep 30
done
