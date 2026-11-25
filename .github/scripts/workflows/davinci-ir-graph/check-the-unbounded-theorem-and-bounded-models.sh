#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/davinci-ir-graph/check-the-unbounded-theorem-and-bounded-models.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
"$RUNNER_TEMP/lean-4.33.1-linux/bin/lean" -DwarningAsError=true Reach.lean | tee "$RUNNER_TEMP/theorem"
grep -Fx "'Davinci.exact_answer' does not depend on any axioms" "$RUNNER_TEMP/theorem"
"$RUNNER_TEMP/lean-4.33.1-linux/bin/lean" -DwarningAsError=true ../position-index/Intervals.lean > "$RUNNER_TEMP/positions"
grep -Fx "'Davinci.PositionIndex.point_eq_scan' depends on axioms: [propext, Classical.choice, Quot.sound]" "$RUNNER_TEMP/positions"
grep -Fx "'Davinci.PositionIndex.selected_rows' depends on axioms: [propext, Classical.choice, Quot.sound]" "$RUNNER_TEMP/positions"
grep -Fx "'Davinci.authored_slice' depends on axioms: [propext, Quot.sound]" "$RUNNER_TEMP/theorem"
grep -Fx "'Davinci.recovery_sites_exact' depends on axioms: [propext, Quot.sound]" "$RUNNER_TEMP/theorem"
java -Djava.awt.headless=true -jar "$RUNNER_TEMP/alloy.jar" exec -q -c '*' -t none -o "$RUNNER_TEMP/instance" Reach.als
jq -e -f "$VIZE_CI_FILES/check-directed-graph.jq" "$RUNNER_TEMP/instance/receipt.json"
grep -Fx "'Davinci.exact_trails' depends on axioms: [propext, Classical.choice, Quot.sound]" "$RUNNER_TEMP/theorem"
java -Djava.awt.headless=true -jar "$RUNNER_TEMP/alloy.jar" exec -q -c '*' -t none -o "$RUNNER_TEMP/trail-instances" Trail.als
jq -e -f "$VIZE_CI_FILES/check-edge-trails.jq" "$RUNNER_TEMP/trail-instances/receipt.json"
java -cp "$RUNNER_TEMP/tlc.jar" tlc2.TLC -workers 2 -metadir "$RUNNER_TEMP/tlc-ok" -config Publication.cfg Publication.tla
result=0
java -cp "$RUNNER_TEMP/tlc.jar" tlc2.TLC -workers 1 -metadir "$RUNNER_TEMP/tlc-negative" -config Stale.cfg Publication.tla > "$RUNNER_TEMP/stale" 2>&1 || result=$?
test "$result" = 12
grep -F 'Invariant CurrentAnswer is violated' "$RUNNER_TEMP/stale"
tlc=(java -cp "$RUNNER_TEMP/tlc.jar" tlc2.TLC -workers 1)
"${tlc[@]}" -metadir "$RUNNER_TEMP/request-model" -config Request.cfg Request.tla
for proof in Version:CurrentRevision Cancelled:LiveRequest Partial:NoPartialOutput; do
  config=${proof%%:*}
  invariant=${proof#*:}
  result=0
  "${tlc[@]}" -metadir "$RUNNER_TEMP/request-$config" -config "Request$config.cfg" Request.tla > "$RUNNER_TEMP/request-$config.log" 2>&1 || result=$?
  test "$result" = 12
  grep -F "Invariant $invariant is violated" "$RUNNER_TEMP/request-$config.log"
done
