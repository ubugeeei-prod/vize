#!/usr/bin/env bash
set -euo pipefail
[[ "$CAPTURE_SOURCE" =~ ^[0-9a-f]{40}$ ]] || exit 1
[[ "$ORIGINAL27_MODE" == capture || "$ORIGINAL27_MODE" == native-props-origin ]] || exit 1
[[ "$NATIVE_VUE_RETURN_OBSERVATION" == false || "$NATIVE_VUE_RETURN_OBSERVATION" == true ]] || exit 1
[[ "$NATIVE_VUE_RETURN_OBSERVATION" == false || "$ORIGINAL27_MODE" == native-props-origin ]] || exit 1
