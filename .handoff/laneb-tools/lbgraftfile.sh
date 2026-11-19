#!/bin/bash
# usage: lbgraftfile.sh <parent> <message> <repo-path>=<local-file>...  — commit file contents onto <parent> without checkout.
set -euo pipefail
cd /Users/nishimura/Source/github.com/ubugeeei-prod/vize/.claude/worktrees/agent-afcf1f8be6bd601bd
P=$1; MSG=$2; shift 2
export GIT_INDEX_FILE=$(mktemp -u /private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/lbidx.XXXXXX)
git read-tree "$P"
for pair in "$@"; do
  path=${pair%%=*}; file=${pair#*=}
  blob=$(git hash-object -w "$file")
  git update-index --add --cacheinfo 100644,"$blob","$path"
done
tree=$(git write-tree)
rm -f "$GIT_INDEX_FILE"
git commit-tree -p "$P" -m "$MSG" "$tree"
