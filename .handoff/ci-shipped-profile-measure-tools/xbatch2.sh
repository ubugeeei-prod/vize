#!/bin/bash
# usage: CRATE=<crate> BASEN=<name> xbatch2.sh name=sha ...   (run from the worktree root)
S=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/pb
for role in "$@"; do
  name=${role%%=*}; sha=${role#*=}
  git checkout -q -f --detach "$sha" || continue
  BASE=/nonexistent bash $S/tools/xmono.sh "$name" "$CRATE" >/dev/null 2>&1
  echo "$name: $(tail -1 $S/rbs/x-$name-mono.txt | cut -c1-50) | vs base: $(python3 $S/tools/cgu2.py $S/rbs/x-$BASEN-mono.txt $S/rbs/x-$name-mono.txt)"
done
echo BATCH_DONE
