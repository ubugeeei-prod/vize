#!/bin/bash
# usage: cmp.sh <base-artifact-dir> <base-role> <roles-dir> role... [-- syms]
# Collects the base role dir and the candidate role dirs into one tmp dir and diffs them.
P=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/pb
BD=$1; B=$2; RD=$3; shift 3
T=$(mktemp -d $P/cmp.XXXXXX)
cp -r "$BD/size-$B" "$T/"
roles=()
for r in "$@"; do cp -r "$RD/size-$r" "$T/"; roles+=("$r"); done
python3 $P/tools/sdiff.py "$T" "$B" "${roles[@]}" --syms ${SYMS:-8}
rm -rf "$T"
