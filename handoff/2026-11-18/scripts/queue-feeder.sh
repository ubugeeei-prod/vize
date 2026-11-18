#!/usr/bin/env bash
# Keep the merge queue at <= MAX entries by enqueueing PRs from the ordered list as earlier ones merge.
# usage: queue-feeder.sh MAX pr1 pr2 ...   Prints one line per enqueue/skip; exits when the list is done.
MAX=$1; shift
TOKEN=$(gh auth token)
gql() { curl -s -H "Authorization: Bearer $TOKEN" -X POST https://api.github.com/graphql -d "$1"; }
pending=("$@")
while [ ${#pending[@]} -gt 0 ]; do
  count=$(gql '{"query":"query{repository(owner:\"ubugeeei-prod\",name:\"vize\"){mergeQueue(branch:\"main\"){entries(first:50){totalCount}}}}"}' | jq -r '.data.repository.mergeQueue.entries.totalCount // 99')
  if [ "$count" -lt "$MAX" ]; then
    pr=${pending[0]}; pending=("${pending[@]:1}")
    info=$(gql "{\"query\":\"query{repository(owner:\\\"ubugeeei-prod\\\",name:\\\"vize\\\"){pullRequest(number:$pr){id state isInMergeQueue mergeable}}}\"}" | jq -r '.data.repository.pullRequest | "\(.id) \(.state) \(.isInMergeQueue) \(.mergeable)"')
    read -r id state inq mergeable <<<"$info"
    if [ "$state" != "OPEN" ] || [ "$inq" = "true" ]; then echo "skip #$pr ($state, inQueue=$inq)"; continue; fi
    res=$(gql "{\"query\":\"mutation{enqueuePullRequest(input:{pullRequestId:\\\"$id\\\"}){mergeQueueEntry{position}}}\"}")
    pos=$(echo "$res" | jq -r '.data.enqueuePullRequest.mergeQueueEntry.position // empty')
    if [ -n "$pos" ]; then echo "enqueued #$pr at $pos ($(date -u +%H:%MZ))"; else echo "FAILED #$pr ($mergeable): $(echo "$res" | jq -c '.errors[0].message')"; fi
    sleep 20; continue
  fi
  sleep 120
done
echo "feeder done $(date -u +%H:%MZ)"
