#!/bin/bash
# usage: xbatch.sh name=sha ...   (run from the worktree root) -> x86 vize_l2 partitions vs x-base
S=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/pb
for role in "$@"; do
  name=${role%%=*}; sha=${role#*=}
  git checkout -q -f --detach "$sha" || continue
  bash $S/tools/xmono.sh "$name" >/dev/null 2>&1
  echo "$name: $(tail -1 $S/rbs/x-$name-mono.txt | cut -c1-60) | vs base: $(python3 $S/tools/cgu2.py $S/rbs/x-base-mono.txt $S/rbs/x-$name-mono.txt) | vs v0: $(python3 $S/tools/cgu2.py $S/rbs/x-v0-mono.txt $S/rbs/x-$name-mono.txt)"
done
echo BATCH_DONE
