#!/usr/bin/env bash
mkdir -p "$SCRIPT_CONTROL_OUTPUT/workflow"
if [[ -d "$RUNNER_TEMP/n8n-script-rule-workflow" ]]; then
  cp -a "$RUNNER_TEMP/n8n-script-rule-workflow/." "$SCRIPT_CONTROL_OUTPUT/workflow/"
fi
printf '{"install":"%s","restore":"%s","pack":"%s","contracts":"%s","controls":"%s"}\n' \
  "$INSTALL_OUTCOME" "$RESTORE_OUTCOME" "$PACK_OUTCOME" "$CONTRACT_OUTCOME" "$CONTROL_OUTCOME" \
  > "$SCRIPT_CONTROL_OUTPUT/workflow/step-outcomes.json"
