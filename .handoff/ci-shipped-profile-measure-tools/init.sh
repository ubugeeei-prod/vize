#!/bin/bash
# usage: init.sh <name>  (run from the worktree root)
# dumps the unmerged per-module CGUs (codegen-units=4096) and the item size estimates
set -e
S=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/pb
CRATE=${CRATE:-vize_l2}
mkdir -p $S/rbs/stats-$1
touch davinci/$CRATE/src/lib.rs crates/$CRATE/src/lib.rs 2>/dev/null || true
nix develop --command bash -c "RUSTC_BOOTSTRAP=1 cargo rustc --locked -p $CRATE --profile ci-opt -- -Zprint-mono-items=yes -Zdump-mono-stats=$S/rbs/stats-$1 -Zdump-mono-stats-format=json -Ccodegen-units=4096" > $S/rbs/$1-init.txt 2>&1
tail -1 $S/rbs/$1-init.txt
