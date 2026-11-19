#!/bin/bash
# wait-merge.sh <pr>: wait until the PR merges, closes, or leaves the merge queue
pr="$1"
for _ in $(seq 1 90); do
  s=$(gh pr view "$pr" --repo ubugeeei-prod/vize --json state --jq .state)
  [ "$s" != OPEN ] && break
  q=$(gh api graphql -f query="{repository(owner:\"ubugeeei-prod\",name:\"vize\"){pullRequest(number:$pr){mergeQueueEntry{state}}}}" --jq '.data.repository.pullRequest.mergeQueueEntry.state // "none"')
  if [ "$q" = none ]; then echo "not in queue"; break; fi
  sleep 30
done
gh pr view "$pr" --repo ubugeeei-prod/vize --json state,mergedAt --jq '.state+" "+(.mergedAt//"")'
