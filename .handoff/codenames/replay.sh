#!/bin/bash
# Replays the codename rename onto a base commit.
# usage: replay.sh <worktree> <base> <branch> <pre-commits-file> <post-commits-file>
#   pre-commits:  script/test commits that precede the move (one sha per line, in order)
#   post-commits: hand fixes that follow the rewrite (one sha per line, in order)
set -euo pipefail
WT=$1; BASE=$2; BRANCH=$3; PRE=$4; POST=$5
RUST_BIN=/nix/store/9sy3gck1w6il739haqr4f7lafyzivkdk-rust-default-1.100.0/bin
cd "$WT"
git cherry-pick --abort 2>/dev/null || true
git reset -q --hard
git clean -fdq
git checkout -q -B "$BRANCH" "$BASE"
git cherry-pick $(grep -v '^#' "$PRE")
node tools/support/levels/rename-codenames.ts --move
git commit -q -m 'refactor(levels): move paths off the codenames (move-only)

Produced by `node tools/support/levels/rename-codenames.ts --move`.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>'
PATH=$RUST_BIN:$PATH node tools/support/levels/rename-codenames.ts --rewrite
git add -A
git commit -q -m 'refactor(levels): rewrite the codenames out of names and text

Produced by `node tools/support/levels/rename-codenames.ts --rewrite`.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>'
git cherry-pick $(grep -v '^#' "$POST")
node tools/support/levels/rename-codenames.ts --check
git log --oneline "$BASE"..HEAD | cat
