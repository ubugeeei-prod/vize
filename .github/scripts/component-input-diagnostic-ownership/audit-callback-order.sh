set -uo pipefail

failed=0
for sdk in actual-20261024 beta-3e923 d6c4; do
  node tools/support/compat/content-mapper/callback-owner-audit.ts \
    "$RUNNER_TEMP/component-input-ownership/callback-cfa/$sdk" || failed=1
done
exit "$failed"
