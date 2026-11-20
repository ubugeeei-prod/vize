set -euo pipefail

capture="$RUNNER_TEMP/component-input-ownership/generic-regression"
mkdir -p "$capture"
cargo test --locked --profile ci -p vize --test integration --no-run --message-format=json \
  > "$capture/build.jsonl" 2> "$capture/build.log"
node .github/scripts/component-input-diagnostic-ownership/retain-generic-repair-observations.ts
