#!/bin/bash
# usage: mkvariants.sh <base-sha> <file-in-repo> <variant-dir> v1 v2 ...  (run from the worktree root)
set -e
BASE=$1; F=$2; R=$3; shift 3
for v in "$@"; do
  git checkout -q --detach "$BASE"
  cp "$R/$v.rs" "$F"
  git commit -q -am "temp: variant $v [temp]"
  echo "$v $(git rev-parse HEAD)"
done
git checkout -q --detach "$BASE"
