#!/bin/sh
# Generated from '../validate-optional-target-sha.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../validate-optional-target-sha.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../validate-optional-target-sha.ush' --checked-in -o 'validate-optional-target-sha.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'validate-optional-target-sha.ush'
__ush_origin='../validate-optional-target-sha.ush'

__ush_at() { __ush_line=$1; __ush_source=$2; }
__ush_report() {
  if [ "$1" -ne 0 ] && [ -n "${__ush_line:-}" ]; then
    __ush_message=${2:-command failed}
    if [ "$#" -lt 2 ]; then
      case "$1" in
        127) __ush_message='command not found; check the command name and PATH' ;;
        126) __ush_message='command cannot be executed; check permissions and interpreter' ;;
        130) __ush_message='interrupted' ;;
        137) __ush_message='process killed (SIGKILL)' ;;
        143) __ush_message='terminated (SIGTERM)' ;;
      esac
    fi
    printf '%s:%s: %s (exit %s)\n  %s\n' "$__ush_origin" "$__ush_line" "$__ush_message" "$1" "$__ush_source" >&2
    __ush_line=''
  fi
}
trap '__ush_report "$?"' 0

case "${1:-}" in
  -h|--help) printf '%s' 'validate-optional-target-sha.ush

Usage:
  validate-optional-target-sha.ush [--help]
  validate-optional-target-sha.ush [--man [ITEM]]
  validate-optional-target-sha.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
validate-optional-target-sha.ush

SYNOPSIS
validate-optional-target-sha.ush [--help]
validate-optional-target-sha.ush [--man [ITEM]]
validate-optional-target-sha.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

ush_fn_std__regex__is_match() {
  VALUE="$1" PATTERN="$2" awk 'BEGIN { value = ENVIRON["VALUE"]; pattern = ENVIRON["PATTERN"]; if (value ~ pattern) { printf "true" } else { printf "false" } }'
}

ush_fn_std__process__exit() {
  case $1 in ''|*[!0-9]*) return 1 ;; esac
  [ "$1" -le 255 ] || return 1
  exit "$1"
}

__ush_at 5 'let requested = $env::REQUESTED_TARGET_SHA'
__ush_env_0=${REQUESTED_TARGET_SHA-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable REQUESTED_TARGET_SHA is not set or is empty'; exit 1; fi
__ush_binding_5_requested="${__ush_env_0}"
# shellcheck disable=SC2034
requested="${__ush_binding_5_requested}"
__ush_at 6 'let run_head = $env::RUN_HEAD_SHA'
__ush_env_1=${RUN_HEAD_SHA-}
if [ -z "${__ush_env_1}" ]; then __ush_report 1 'environment variable RUN_HEAD_SHA is not set or is empty'; exit 1; fi
__ush_binding_6_run_head="${__ush_env_1}"
# shellcheck disable=SC2034
run_head="${__ush_binding_6_run_head}"
__ush_at 8 'if !(is_match $requested "^[0-9a-f]{40}$") {'
if ! {
__ush_value_2="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__regex__is_match "${__ush_binding_5_requested}" '^[0-9a-f]{40}$')" || exit "$?"
[ "${__ush_value_2}" = 'true' ]
}; then
  __ush_at 9 'echo "::error title=Invalid target_sha::target_sha must be a full lowercase 40-character commit SHA"'
  printf '%s\n' '::error title=Invalid target_sha::target_sha must be a full lowercase 40-character commit SHA'
  __ush_at 10 'exit 1'
  ush_fn_std__process__exit 1
fi
__ush_at 12 'if run_head != requested {'
if {
[ "$(if [ "${__ush_binding_6_run_head}" != "${__ush_binding_5_requested}" ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 13 'echo $"::error title=Dispatch ref mismatch::--ref must point at $requested (got $run_head)"'
  printf '%s\n' '::error title=Dispatch ref mismatch::--ref must point at '"${__ush_binding_5_requested}"' (got '"${__ush_binding_6_run_head}"')'
  __ush_at 14 'exit 1'
  ush_fn_std__process__exit 1
fi
