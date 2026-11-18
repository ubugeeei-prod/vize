#!/usr/bin/env bash
# Poll every 2 min; print new third-party issues/PRs/comments since start (or $1) and exit after the first batch.
TOKEN=$(gh auth token)
since=${1:-$(date -u +%Y-%m-%dT%H:%M:%SZ)}
while true; do
  now=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  out=$(
    curl -s -H "Authorization: Bearer $TOKEN" "https://api.github.com/repos/ubugeeei-prod/vize/issues?state=all&since=$since&per_page=50&sort=created&direction=desc" 2>/dev/null \
      | jq -r --arg s "$since" '.[]? | select(.user.login!="ubugeeei" and (.user.type!="Bot") and .created_at >= $s) | "NEW \(if .pull_request then "PR" else "ISSUE" end) #\(.number) by \(.user.login): \(.title)"' 2>/dev/null
    curl -s -H "Authorization: Bearer $TOKEN" "https://api.github.com/repos/ubugeeei-prod/vize/issues/comments?since=$since&per_page=50" 2>/dev/null \
      | jq -r '.[]? | select(.user.login!="ubugeeei" and (.user.type!="Bot") and (.user.login|test("coderabbit|github-actions|copilot";"i")|not)) | "COMMENT #\(.issue_url|split("/")|last) by \(.user.login): \(.body|gsub("\n";" ")|.[0:200])"' 2>/dev/null
  )
  if [ -n "$out" ]; then echo "$out"; echo "next-since=$now"; exit 0; fi
  since=$now
  sleep 120
done
