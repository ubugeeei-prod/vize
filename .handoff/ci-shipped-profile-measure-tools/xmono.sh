#!/bin/bash
# usage: xmono.sh <name> [crate]  (run from the worktree root)
# x86_64-unknown-linux-gnu ci-opt partition of one crate; compares with $BASE (default rbs/x-base-mono.txt)
set -e
S=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/pb
CRATE=${2:-vize_l2}
BASE=${BASE:-$S/rbs/x-base-mono.txt}
touch davinci/$CRATE/src/lib.rs crates/$CRATE/src/lib.rs 2>/dev/null || true
nix develop --command bash -c "RUSTC_BOOTSTRAP=1 RUSTFLAGS='--sysroot=$S/sysroot -C target-cpu=x86-64' CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/true CARGO_TARGET_DIR=target/x86 cargo rustc --locked -p $CRATE --target x86_64-unknown-linux-gnu --profile ci-opt -- -Zprint-mono-items=yes" > $S/rbs/x-$1-mono.txt 2>&1
tail -1 $S/rbs/x-$1-mono.txt
[ -f "$BASE" ] && python3 $S/tools/cgu.py $BASE $S/rbs/x-$1-mono.txt 2>/dev/null | head -2
true
