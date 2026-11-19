#!/bin/bash
# usage: replay-plan.sh <repo> <base> <branch> <pre-out> <post-out>
# Splits a rename branch into the commits a replay re-applies: pre-move commits (everything before
# the move commit, and any later commit touching only the rename script and its test) and
# post-rewrite commits (the rest after the rewrite). Drops the move and rewrite commits themselves
# and any "tmp:" commit (data the script reproduces).
cd "$1" || exit 2
BASE=$2; BRANCH=$3; PRE=$4; POST=$5
: > "$PRE"; : > "$POST"
phase=pre
git log --reverse --format='%h%x09%s' "$BASE..$BRANCH" | while IFS=$'\t' read -r sha subject; do
  case "$subject" in
    "refactor(levels): move paths off the codenames (move-only)") phase=moved; continue ;;
    "refactor(levels): rewrite the codenames out of names and text") phase=post; continue ;;
    tmp:*) continue ;;
  esac
  if [ "$phase" = pre ]; then echo "$sha" >> "$PRE"; continue; fi
  files=$(git show --format= --name-only "$sha" | sort -u | tr '\n' ' ')
  if [ "$files" = "tools/support/levels/rename-codenames.ts " ] ||
     [ "$files" = "tests/tooling/codename-rename.test.ts tools/support/levels/rename-codenames.ts " ]; then
    echo "$sha" >> "$PRE"
  else
    echo "$sha" >> "$POST"
  fi
done
wc -l "$PRE" "$POST"
