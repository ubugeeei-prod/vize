#!/bin/sh
git() {
  if [ "$1" = "branch" ]; then printf '%s\n' feature/testbox; fi
}
vp() {
  printf '%s|id=%s\n' "$*" "${BLACKSMITH_TESTBOX_ID-unset}" >> "$FAKE_LIFECYCLE_LOG"
  case "$*" in
    "run --workspace-root testbox:warmup")
      if [ "$FAKE_WARMUP_STATUS" -eq 0 ]; then printf '%s\n' fresh-box; fi
      return "$FAKE_WARMUP_STATUS" ;;
    "run --workspace-root build:testbox") return "${FAKE_TASK_STATUS:-0}" ;;
    "run --workspace-root testbox:stop") return "${FAKE_STOP_STATUS:-0}" ;;
  esac
}
. "$1"
lifecycle_status=$?
printf 'final|id=%s\n' "${BLACKSMITH_TESTBOX_ID-unset}" >> "$FAKE_LIFECYCLE_LOG"
exit "$lifecycle_status"
