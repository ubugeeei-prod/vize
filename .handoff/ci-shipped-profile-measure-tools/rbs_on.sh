#!/bin/bash
# usage: rbs_on.sh <parent> <variant.rs> <message>  -> commit of the #13340 change (variant text_facts) on <parent>
set -e
P=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/pb
D=$P/rbs/pr-files
mkdir -p $D
SRC=d2de96ee8c46044cf7ef4a03487ef7e2199b5e44
for f in .github/workflows/fuzz.yml davinci/vize_l2/tests/text_rebuild.rs tests/fuzz/Cargo.toml tests/fuzz/fuzz_targets/text_rebuild.rs tools/commands/ci/fuzz/seed-corpus.ts; do
  mkdir -p "$D/$(dirname $f)"
  git show "$SRC:$f" > "$D/$f"
done
MSG="$3"; [ -f "$3" ] && MSG="$(cat "$3")"
bash $P/tools/plumb.sh "$1" "$MSG" \
  davinci/vize_l2/src/analysis/text_facts.rs "$2" \
  .github/workflows/fuzz.yml $D/.github/workflows/fuzz.yml \
  davinci/vize_l2/tests/text_rebuild.rs $D/davinci/vize_l2/tests/text_rebuild.rs \
  tests/fuzz/Cargo.toml $D/tests/fuzz/Cargo.toml \
  tests/fuzz/fuzz_targets/text_rebuild.rs $D/tests/fuzz/fuzz_targets/text_rebuild.rs \
  tools/commands/ci/fuzz/seed-corpus.ts $D/tools/commands/ci/fuzz/seed-corpus.ts
