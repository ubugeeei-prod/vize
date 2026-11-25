#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/canonical-corpus-divergence/file-canonical-divergence-issue.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
args=(publish "$OBSERVER" "real-project-$OBSERVER-corpus" "real-project-$OBSERVER-reduction")
if [[ -s "real-project-$OBSERVER-promotion/promotion.json" ]]; then
  args+=("real-project-$OBSERVER-promotion")
fi
node tools/support/compat/github/canonical-corpus-divergence-issue.mjs "${args[@]}"
