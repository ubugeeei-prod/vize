#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-small-node-cases/scripts/run-selected-node-tests.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
case "$SMALL_NODE_SUITE" in
  check-js)
    vp run --workspace-root check:iteration
    tests=(
      tests/tooling/github-workflows-source-selection.test.ts
      tests/tooling/vp-bootstrap-cache.test.mjs
    )
    ;;
  node-engine)
    tests=(
      tests/tooling/node-engine-matrix.test.ts
      tests/tooling/package-manifests.test.ts
      tests/tooling/entry-packages.test.ts
      tests/tooling/bundler-plugin-manifests.test.ts
      tests/tooling/dependency-security-compat.test.ts
      tests/tooling/comparison-base.test.ts
      tests/tooling/plan-source-checks.test.ts
      tests/tooling/plan-check-inputs.test.ts
      tests/tooling/warning-budget.test.ts
      tests/tooling/npm-audit-command.test.ts
    )
    ;;
  *) echo "unknown small Node suite: $SMALL_NODE_SUITE" >&2; exit 2 ;;
esac
node tools/support/compat/github/js-test-cases.ts capture -- node --test "${tests[@]}"
