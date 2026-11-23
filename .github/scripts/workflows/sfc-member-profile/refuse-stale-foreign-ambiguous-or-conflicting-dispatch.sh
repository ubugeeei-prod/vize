#!/usr/bin/env bash
set -euo pipefail
[[ "$MEMBER_BASE_SHA" =~ ^[0-9a-f]{40}$ && "$MEMBER_HEAD_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$MEMBER_HEAD_SHA" == "$GITHUB_SHA" && "$MEMBER_CONFLICTING_MODES" == false ]]
[[ "$GITHUB_REPOSITORY" == ubugeeei-prod/vize ]]
[[ "$(git -C head rev-parse HEAD)" == "$MEMBER_HEAD_SHA" ]]
git -C head merge-base --is-ancestor "$MEMBER_BASE_SHA" "$MEMBER_HEAD_SHA"
