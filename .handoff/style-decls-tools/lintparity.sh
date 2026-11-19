#!/bin/bash
# usage: lintparity.sh <name> <vize-binary>   -> writes outputs under style/lint/<name>/
set -u
S=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/pb/style
G=/Users/nishimura/Source/github.com/ubugeeei-prod/vize/tests/_fixtures/_git
O=$S/lint/$1
mkdir -p $O
for corpus in misskey n8n; do
  for preset in happy-path opinionated nuxt; do
    (cd $G/$corpus && RAYON_NUM_THREADS=1 "$2" lint . --no-config --preset $preset --format json) > $O/$corpus-$preset.json 2> $O/$corpus-$preset.err
    echo "$corpus $preset exit=$? bytes=$(wc -c < $O/$corpus-$preset.json)"
  done
done
