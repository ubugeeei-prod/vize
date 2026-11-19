#!/bin/bash
# usage: lbgraft.sh <parent> <source-commit> <message> <path>...
# Prints a commit on <parent> whose tree is <parent>'s with <path>s taken from <source-commit>.
set -euo pipefail
cd /Users/nishimura/Source/github.com/ubugeeei-prod/vize/.claude/worktrees/agent-afcf1f8be6bd601bd
P=$1; SRC=$2; MSG=$3; shift 3
export GIT_INDEX_FILE=$(mktemp -u /private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/lbidx.XXXXXX)
git read-tree "$P"
for path in "$@"; do
  blob=$(git rev-parse "$SRC:$path")
  git update-index --add --cacheinfo 100644,"$blob","$path"
done
tree=$(git write-tree)
rm -f "$GIT_INDEX_FILE"
git commit-tree -p "$P" -m "$MSG" "$tree"
