#!/usr/bin/env bash
set -euo pipefail
for row in vue26:2.6.14 vue27:2.7.16; do
  name="${row%%:*}"
  version="${row#*:}"
  directory="$VIZE_VUE2_IR_ORACLES_ROOT/$name"
  mkdir -p "$directory"
  printf '{"name":"vize-vue2-ir-reference","private":true,"version":"1.0.0"}\n' > "$directory/package.json"
  npm --prefix "$directory" install --save-exact --ignore-scripts --no-audit --no-fund "vue@$version" "vue-template-compiler@$version" "vue-server-renderer@$version" > "target/vize-tests/vue2-ir/$name.install.stdout" 2> "target/vize-tests/vue2-ir/$name.install.stderr"
  cp "$directory/package-lock.json" "target/vize-tests/vue2-ir/$name.package-lock.json"
done
