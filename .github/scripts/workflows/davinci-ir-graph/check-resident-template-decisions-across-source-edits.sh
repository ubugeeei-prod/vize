#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/davinci-ir-graph/check-resident-template-decisions-across-source-edits.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
"$RUNNER_TEMP/lean-4.33.1-linux/bin/lean" -DwarningAsError=true TemplateProjection.lean | tee "$RUNNER_TEMP/template-projection"
grep -Fx "'Davinci.TemplateProjection.exact' depends on axioms: [propext]" "$RUNNER_TEMP/template-projection"
grep -Fx "'Davinci.TemplateProjection.endpoints_fit' does not depend on any axioms" "$RUNNER_TEMP/template-projection"
tlc=(java -cp "$RUNNER_TEMP/tlc.jar" tlc2.TLC -workers 1)
"${tlc[@]}" -metadir "$RUNNER_TEMP/template-revision" -config TemplateRevision.cfg TemplateRevision.tla
for defect in Content Base Duplicate; do
  result=0
  "${tlc[@]}" -metadir "$RUNNER_TEMP/template-$defect" -config "TemplateRevision$defect.cfg" TemplateRevision.tla > "$RUNNER_TEMP/template-$defect.log" 2>&1 || result=$?
  test "$result" = 12
  grep -F 'Invariant ExactReport is violated' "$RUNNER_TEMP/template-$defect.log"
done
