#!/usr/bin/env bash
if [[ "$PROBE_OUTCOME" != success || "$AUTOMATIC_OUTCOME" != success ]]; then
  echo 'Preview discovery or a failed control cannot qualify source support.' >&2
  exit 1
fi
