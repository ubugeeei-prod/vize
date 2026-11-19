#!/bin/bash
# usage: lbrelcommit.sh <commit> <message>  -> prints a commit on top of <commit> whose ci-opt profile is fat LTO / 1 CGU
set -euo pipefail
cd /Users/nishimura/Source/github.com/ubugeeei-prod/vize/.claude/worktrees/agent-afcf1f8be6bd601bd
C=$1; MSG=$2
blob=$(git show "$C:Cargo.toml" | python3 -c '
import sys
s=sys.stdin.read()
old="[profile.ci-opt]\ninherits = \"release\"\nlto = \"thin\"\ncodegen-units = 16\n"
assert old in s
sys.stdout.write(s.replace(old,"[profile.ci-opt]\ninherits = \"release\"\nlto = \"fat\"\ncodegen-units = 1\n"))' | git hash-object -w --stdin)
export GIT_INDEX_FILE=$(mktemp -u /private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/lbidx.XXXXXX)
git read-tree "$C"
git update-index --cacheinfo 100644,"$blob",Cargo.toml
tree=$(git write-tree)
rm -f "$GIT_INDEX_FILE"
git commit-tree -p "$C" -m "$MSG" "$tree"
