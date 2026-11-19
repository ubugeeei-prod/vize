#!/bin/bash
# usage: mono.sh <name>  (run from the worktree root) -> builds vize_l2 mono items and compares with base
set -e
S=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/pb
CRATE=${CRATE:-vize_l2}
BASE=${BASE:-$S/rbs/base-mono.txt}
touch davinci/$CRATE/src/lib.rs crates/$CRATE/src/lib.rs 2>/dev/null || true
nix develop --command bash -c "RUSTC_BOOTSTRAP=1 cargo rustc --locked -p $CRATE --profile ci-opt -- -Zprint-mono-items=yes" > $S/rbs/$1-mono.txt 2>&1
tail -1 $S/rbs/$1-mono.txt
python3 $S/tools/cgu.py $BASE $S/rbs/$1-mono.txt 2>/dev/null | head -2
