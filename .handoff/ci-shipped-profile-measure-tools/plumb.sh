#!/bin/bash
# usage: plumb.sh <parent-sha> <message> <repo-path> <file> [<repo-path> <file> ...]
# Creates a commit on top of <parent-sha> replacing the given files, without touching the worktree.
set -e
PARENT=$1; MSG=$2; shift 2
export GIT_INDEX_FILE=$(mktemp -u /private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/pb/idx.XXXXXX)
git read-tree "$PARENT"
while [ $# -gt 0 ]; do
  blob=$(git hash-object -w "$2")
  git update-index --add --cacheinfo "100644,$blob,$1"
  shift 2
done
tree=$(git write-tree)
rm -f "$GIT_INDEX_FILE"
git commit-tree "$tree" -p "$PARENT" -m "$MSG"
