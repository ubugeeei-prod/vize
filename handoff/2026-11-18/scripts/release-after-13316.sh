#!/usr/bin/env bash
# Unattended: wait for #13316 to merge, then release main HEAD once its Check (push or merge_group, same SHA)
# succeeded, wait for the release, and post "Fixed in" on #13284 when the version is live on npm.
set -u
R=ubugeeei-prod/vize
log() { echo "[$(date -u +%H:%MZ)] $*"; }
until [ "$(gh pr view 13316 -R $R --json state --jq .state 2>/dev/null)" = "MERGED" ]; do sleep 120; done
log "#13316 merged"
sleep 60
SHA=$(gh api repos/$R/commits/main --jq .sha)
log "main HEAD $SHA"
ok=""
for i in $(seq 1 120); do
  ok=$(gh api "repos/$R/actions/runs?head_sha=$SHA&per_page=50" --jq '[.workflow_runs[] | select((.name|startswith("Check push") or startswith("Check merge_group")) and .conclusion=="success")] | length' 2>/dev/null)
  [ "${ok:-0}" -gt 0 ] && break
  sleep 120
done
[ "${ok:-0}" -gt 0 ] || { log "no successful Check for $SHA; not releasing"; exit 1; }
gh workflow run release.yml -R $R --ref main -f source="$SHA" || { log "dispatch failed"; exit 1; }
sleep 20
RUN=$(gh run list -R $R --workflow release.yml --limit 1 --json databaseId --jq '.[0].databaseId')
log "release run $RUN dispatched at $SHA"
until [ "$(gh run view "$RUN" -R $R --json status --jq .status)" = "completed" ]; do sleep 120; done
CONC=$(gh run view "$RUN" -R $R --json conclusion --jq .conclusion)
log "release run $RUN: $CONC"
[ "$CONC" = "success" ] || exit 1
TAG=$(gh release list -R $R --limit 1 --json tagName --jq '.[0].tagName')
VER=${TAG#v}
for i in $(seq 1 30); do
  [ "$(curl -s https://registry.npmjs.org/vize | jq -r --arg v "$VER" '.versions[$v].version // empty')" = "$VER" ] && break
  sleep 60
done
gh issue comment 13284 -R $R --body "Fixed in [${TAG}](https://github.com/$R/releases/tag/${TAG}) (#13288)." && log "commented Fixed in $TAG on #13284"
