#!/bin/bash
# usage: renorm.sh <parity workdir>  -> recompare the logs with output paths normalized
W=$1
for m in dom ssr vapor; do
  for role in base head; do
    sed -E 's/"(elapsed|duration|time|ms|timing)[A-Za-z_]*": *[0-9.]+/"\1":0/g; s/[0-9]+(\.[0-9]+)?ms/Nms/g; s/out-(base|head)-/out-X-/g; /slow file\(s\) detected/,$d' "$W/log-$role-$m.txt" > "$W/log-$role-$m.norm"
  done
  if cmp -s "$W/log-base-$m.norm" "$W/log-head-$m.norm"; then
    echo "$m: diagnostics identical ($(grep -c . "$W/log-base-$m.norm") log lines, $(grep -c -i -E 'error' "$W/log-base-$m.norm") error lines)"
  else
    echo "$m: DIAGNOSTICS DIFFER"; diff "$W/log-base-$m.norm" "$W/log-head-$m.norm" | head -5
  fi
done
