#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/declaration-source-products/model-native-diagnostic-publication.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
jar="$RUNNER_TEMP/ambient-tlc.jar"
curl -fsSL --retry 3 -o "$jar" https://github.com/tlaplus/tlaplus/releases/download/v1.7.4/tla2tools.jar
echo "936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88  $jar" | sha256sum -c -
tlc=(java -cp "$jar" tlc2.TLC -workers 1)
"${tlc[@]}" -metadir "$RUNNER_TEMP/ambient-model" -config Publication.cfg Publication.tla
for fault in ack native doc sources host revision publish; do
  cfg="$RUNNER_TEMP/ambient-$fault.cfg"
  awk -v fault="$fault" -f "$VIZE_CI_FILES/ambient-publication-fault.awk" Publication.cfg > "$cfg"
  result=0
  "${tlc[@]}" -metadir "$RUNNER_TEMP/ambient-$fault" -config "$cfg" Publication.tla > "$RUNNER_TEMP/ambient-$fault.log" 2>&1 || result=$?
  test "$result" = 12
  grep -F 'Invariant ExactPublication is violated' "$RUNNER_TEMP/ambient-$fault.log"
done
