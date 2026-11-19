#!/bin/bash
# usage: parity.sh <base-vize> <head-vize> <workdir>   (run from the worktree root)
# Compiles every repo .vue / .vue.txt input in DOM, SSR and Vapor mode with both
# binaries and diffs the emitted files plus the JSON diagnostics.
set -u
BASE=$1; HEAD=$2; W=$3
mkdir -p "$W"
if [ ! -d "$W/corpus" ]; then
  mkdir -p "$W/corpus"
  git ls-files -z '*.vue' '*.vue.txt' | while IFS= read -r -d '' f; do
    d="$W/corpus/$(dirname "$f")"
    mkdir -p "$d"
    cp "$f" "$d/$(basename "$f" .txt)"
  done
fi
echo "inputs: $(find "$W/corpus" -name '*.vue' | wc -l)"
status=0
for mode in dom ssr vapor; do
  flag=""
  [ $mode = ssr ] && flag=--ssr
  [ $mode = vapor ] && flag=--vapor
  for role in base head; do
    bin=$BASE; [ $role = head ] && bin=$HEAD
    rm -rf "$W/out-$role-$mode"
    (cd "$W/corpus" && RAYON_NUM_THREADS=1 "$bin" build '**/*.vue' -o "$W/out-$role-$mode" --no-config $flag --continue-on-error -j 1 --format json) \
      > "$W/log-$role-$mode.txt" 2>&1
    echo "$role $mode exit=$?" >> "$W/exits.txt"
  done
  files=$(find "$W/out-base-$mode" -type f | wc -l)
  if diff -r "$W/out-base-$mode" "$W/out-head-$mode" > "$W/diff-$mode.txt"; then
    echo "$mode: outputs identical ($files files)"
  else
    echo "$mode: OUTPUTS DIFFER ($(wc -l < "$W/diff-$mode.txt") diff lines)"; status=1
  fi
  # diagnostics: drop timing fields
  for role in base head; do
    sed -E 's/"(elapsed|duration|time|ms|timing)[A-Za-z_]*": *[0-9.]+/"\1":0/g; s/[0-9]+(\.[0-9]+)?ms/Nms/g; s/out-(base|head)-/out-X-/g' "$W/log-$role-$mode.txt" > "$W/log-$role-$mode.norm"
  done
  if cmp -s "$W/log-base-$mode.norm" "$W/log-head-$mode.norm"; then
    echo "$mode: diagnostics identical ($(wc -c < "$W/log-base-$mode.norm") bytes of log)"
  else
    echo "$mode: DIAGNOSTICS DIFFER"; status=1
  fi
done
exit $status
