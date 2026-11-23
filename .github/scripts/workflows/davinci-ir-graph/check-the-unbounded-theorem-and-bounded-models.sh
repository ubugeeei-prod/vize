#!/usr/bin/env bash
set -euo pipefail
"$RUNNER_TEMP/lean-4.33.1-linux/bin/lean" -DwarningAsError=true Reach.lean | tee "$RUNNER_TEMP/theorem"
grep -Fx "'Davinci.exact_answer' does not depend on any axioms" "$RUNNER_TEMP/theorem"
java -Djava.awt.headless=true -jar "$RUNNER_TEMP/alloy.jar" exec -q -c '*' -t none -o "$RUNNER_TEMP/instance" Reach.als
jq -e '.commands | keys == ["DisconnectedCycle","Exact","MissedSuccessor","ParallelCycle"] and (.Exact | .type == "check" and (.solution | length) == 0) and ([.DisconnectedCycle,.MissedSuccessor,.ParallelCycle] | all(.type == "run" and (.solution | length) == 1))' "$RUNNER_TEMP/instance/receipt.json"
grep -Fx "'Davinci.exact_trails' depends on axioms: [propext, Classical.choice, Quot.sound]" "$RUNNER_TEMP/theorem"
java -Djava.awt.headless=true -jar "$RUNNER_TEMP/alloy.jar" exec -q -c '*' -t none -o "$RUNNER_TEMP/trail-instances" Trail.als
jq -e '.commands | keys == ["DistinctSelfLoops","ExactTrails","ParallelReturn","SingleEdgeReturnsOnlyOnReuse"] and (.ExactTrails | .type == "check" and (.solution | length) == 0) and ([.DistinctSelfLoops,.ParallelReturn,.SingleEdgeReturnsOnlyOnReuse] | all(.type == "run" and (.solution | length) == 1))' "$RUNNER_TEMP/trail-instances/receipt.json"
java -cp "$RUNNER_TEMP/tlc.jar" tlc2.TLC -workers 2 -metadir "$RUNNER_TEMP/tlc-ok" -config Publication.cfg Publication.tla
result=0
java -cp "$RUNNER_TEMP/tlc.jar" tlc2.TLC -workers 1 -metadir "$RUNNER_TEMP/tlc-negative" -config Stale.cfg Publication.tla > "$RUNNER_TEMP/stale" 2>&1 || result=$?
test "$result" = 12
grep -F 'Invariant CurrentAnswer is violated' "$RUNNER_TEMP/stale"
