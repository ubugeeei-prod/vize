#!/bin/bash
# Exit with a line when any listed PR merges/closes, or is neither queued nor armed, or has a failed check.
cd /Users/ubugeeei/Source/github.com/ubugeeei-prod/vize
while true; do
  for p in "$@"; do
    s=$(gh api graphql -f query='query($n:Int!){repository(owner:"ubugeeei-prod",name:"vize"){pullRequest(number:$n){state isInMergeQueue autoMergeRequest{enabledAt} commits(last:1){nodes{commit{statusCheckRollup{contexts(first:100){nodes{... on CheckRun{name conclusion}}}}}}}}}}' -F n=$p --jq '.data.repository.pullRequest | "\(.state) \(.isInMergeQueue) \(.autoMergeRequest != null) " + ([.commits.nodes[0].commit.statusCheckRollup.contexts.nodes[] | select(.conclusion=="FAILURE") | .name] | join("|"))' 2>/dev/null) || continue
    read -r state queued auto fails <<< "$s"
    if [ "$state" != "OPEN" ]; then echo "PR $p $state"; exit 0; fi
    if [ "$queued" != "true" ] && [ "$auto" != "true" ]; then echo "PR $p neither queued nor armed; fails=$fails"; exit 0; fi
    if [ -n "$fails" ]; then echo "PR $p failing: $fails"; exit 0; fi
  done
  sleep 90
done
