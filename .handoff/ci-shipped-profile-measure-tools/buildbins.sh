#!/bin/bash
# usage: buildbins.sh name=sha ...  (run from the worktree root) -> scratch/bin/<name>-vize (ci profile)
S=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/pb
mkdir -p $S/bin
for role in "$@"; do
  name=${role%%=*}; sha=${role#*=}
  git checkout -q -f --detach "$sha" || { echo "$name checkout failed"; continue; }
  nix develop --command bash -c 'cargo build --locked -p vize --profile ci 2>&1 | tail -1'
  cp target/ci/vize $S/bin/$name-vize && echo "$name built $(shasum -a 256 $S/bin/$name-vize | cut -c1-16)"
done
echo BUILD_DONE
