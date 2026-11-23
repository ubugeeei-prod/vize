#!/usr/bin/env bash
mkdir -p "$OBSERVER_OUTPUT/workflow"
if [[ -d "$RUNNER_TEMP/n8n-whole-corpus-workflow" ]]; then
  cp -a "$RUNNER_TEMP/n8n-whole-corpus-workflow/." "$OBSERVER_OUTPUT/workflow/"
fi
printf '{"install":"%s","hydrate":"%s","build":"%s","pack":"%s","observe":"%s"}\n' \
  "$INSTALL_OUTCOME" "$HYDRATE_OUTCOME" "$BUILD_OUTCOME" "$PACK_OUTCOME" "$OBSERVE_OUTCOME" \
  > "$OBSERVER_OUTPUT/workflow/step-outcomes.json"
