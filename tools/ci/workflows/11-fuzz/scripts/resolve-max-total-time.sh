#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/11-fuzz/scripts/resolve-max-total-time.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ "$FUZZ_MODE" == "replay" ]]; then
  # `-runs=0` executes every corpus input once and exits, so no budget
  # applies and the verdict depends only on the corpus.
  echo "seconds=0" >> "$GITHUB_OUTPUT"
  exit 0
fi
case "${GITHUB_EVENT_NAME}" in
  schedule)
    # Nightly long fuzz: 30 minutes/target — large enough to drive
    # libFuzzer past coverage plateaus on the seeded corpus.
    seconds=1800
    ;;
  workflow_dispatch)
    seconds="$REQUESTED_MAX_TOTAL_TIME"
    ;;
  *)
    echo "::error title=Unsupported fuzz event::campaign requires schedule or workflow_dispatch"
    exit 1
    ;;
esac
if [[ ! "$seconds" =~ ^[0-9]{1,4}$ ]] || (( 10#$seconds < 1 || 10#$seconds > 3600 )); then
  echo "::error title=Invalid fuzz budget::max-total-time must be an integer from 1 to 3600 seconds"
  exit 1
fi
echo "seconds=$seconds" >> "$GITHUB_OUTPUT"
