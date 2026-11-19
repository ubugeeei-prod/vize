#!/bin/bash
# usage: lbparity_cmp.sh <label> — compare a finished lbparity.sh run with normalized, sorted diagnostics.
S=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad
OUT=$S/laneb-parity-$1
# Timing-dependent "slow file" report lines are excluded; every diagnostic line is kept.
norm() { sed -E -e "s#$OUT/(base|head)/#OUT/#g" -e 's/compiled in [0-9.]+(ms|s|µs)/compiled in T/' "$1" | grep -vE '→ Large (script|template)|→ .* is slow|\(parse: |^  \.\.\. and [0-9]+ more|slow file\(s\) detected' | sort; }
for mode in dom ssr vapor; do
  for k in json js; do
    files=$(find "$OUT/base/$k-$mode" -type f | wc -l | tr -d ' ')
    out=same; diff -rq "$OUT/base/$k-$mode" "$OUT/head/$k-$mode" > /dev/null || out=DIFF
    err=same; diff -q <(norm "$OUT/base/$k-$mode.stderr") <(norm "$OUT/head/$k-$mode.stderr") > /dev/null || err=DIFF
    ex=same; diff -q "$OUT/base/$k-$mode.stdout" "$OUT/head/$k-$mode.stdout" > /dev/null || ex=DIFF
    diags=$(grep -c "" "$OUT/base/$k-$mode.stderr")
    echo "$mode $k: outputs=$out ($files files) diagnostics=$err ($diags lines) exit=$ex"
    [ "$err" = DIFF ] && [ -n "$VERBOSE" ] && diff <(norm "$OUT/base/$k-$mode.stderr") <(norm "$OUT/head/$k-$mode.stderr") | head -8
  done
done
