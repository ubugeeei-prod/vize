#!/bin/bash
# usage: lbparity.sh <base-bin> <head-bin> <label>
# Copies the worktree's tracked .vue/.tsrx files into a corpus and compiles it with both binaries
# in DOM, SSR and Vapor modes (json + js), then diffs outputs and diagnostics.
S=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad
W=/Users/nishimura/Source/github.com/ubugeeei-prod/vize/.claude/worktrees/agent-afcf1f8be6bd601bd
BASE=$1; HEAD=$2; L=$3
OUT=$S/laneb-parity-$L; rm -rf "$OUT"; mkdir -p "$OUT/corpus"
cd "$W" && git ls-files -z '*.vue' '*.tsrx' | (cd "$W" && xargs -0 -I{} rsync -R "{}" "$OUT/corpus/")
N=$(find "$OUT/corpus" -name '*.vue' -o -name '*.tsrx' | wc -l | tr -d ' ')
echo "files $N"
cd "$OUT/corpus" || exit 2
for side in base head; do
  BIN=$BASE; [ $side = head ] && BIN=$HEAD; mkdir -p "$OUT/$side"
  for mode in dom ssr vapor; do
    flag=""; [ $mode = ssr ] && flag=--ssr; [ $mode = vapor ] && flag=--vapor
    "$BIN" build '**/*.vue' --no-config $flag -f json -o "$OUT/$side/json-$mode" -j 8 --continue-on-error > "$OUT/$side/json-$mode.stdout" 2> "$OUT/$side/json-$mode.stderr"; echo "exit $?" >> "$OUT/$side/json-$mode.stdout"
    "$BIN" build '**/*.vue' --no-config $flag -o "$OUT/$side/js-$mode" -j 8 --continue-on-error > "$OUT/$side/js-$mode.stdout" 2> "$OUT/$side/js-$mode.stderr"; echo "exit $?" >> "$OUT/$side/js-$mode.stdout"
  done 2>/dev/null
done
for mode in dom ssr vapor; do
  for k in json js; do
    files=$(find "$OUT/base/$k-$mode" -type f | wc -l | tr -d ' ')
    if diff -rq "$OUT/base/$k-$mode" "$OUT/head/$k-$mode" > /dev/null && diff -q <(sed -E 's/[0-9.]+ ?(ms|s|µs)//g' "$OUT/base/$k-$mode.stderr") <(sed -E 's/[0-9.]+ ?(ms|s|µs)//g' "$OUT/head/$k-$mode.stderr") >/dev/null; then
      echo "$mode $k: identical ($files outputs)"
    else
      echo "$mode $k: DIFFERS"; diff -rq "$OUT/base/$k-$mode" "$OUT/head/$k-$mode" | head -5
    fi
  done
done
echo EXIT
