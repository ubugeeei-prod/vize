#!/bin/sh
# Generated from '../refuse-a-stale-or-different-dispatch-source.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../refuse-a-stale-or-different-dispatch-source.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../refuse-a-stale-or-different-dispatch-source.ush' --checked-in -o 'refuse-a-stale-or-different-dispatch-source.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'refuse-a-stale-or-different-dispatch-source.ush'
__ush_origin='../refuse-a-stale-or-different-dispatch-source.ush'

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
  -h|--help) printf '%s' 'refuse-a-stale-or-different-dispatch-source.ush

Usage:
  refuse-a-stale-or-different-dispatch-source.ush [--help]
  refuse-a-stale-or-different-dispatch-source.ush [--man [ITEM]]
  refuse-a-stale-or-different-dispatch-source.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
refuse-a-stale-or-different-dispatch-source.ush

SYNOPSIS
refuse-a-stale-or-different-dispatch-source.ush [--help]
refuse-a-stale-or-different-dispatch-source.ush [--man [ITEM]]
refuse-a-stale-or-different-dispatch-source.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

exec 3>&1

ush_fn_is_commit_sha() {
  __ush_fn_is_commit_sha_arg_0="${1}"
  # shellcheck disable=SC2034
  value="${__ush_fn_is_commit_sha_arg_0}"
  __ush_at 6 'match value {'
  case "${__ush_fn_is_commit_sha_arg_0}" in
    *[!0123456789abcdef]*)
      __ush_at 7 'glob "*[!0-9a-f]*" => false'
      if [ -n "${__ush_return_path:-}" ]; then
        printf '%s' 'false' > "$__ush_return_path"
      elif [ "${__ush_capture_return:-0}" = '1' ]; then
        printf '%s' 'false' >&4
      fi
      return 0
      ;;
    ????????????????????????????????????????)
      __ush_at 8 'glob "????????????????????????????????????????" => true'
      if [ -n "${__ush_return_path:-}" ]; then
        printf '%s' 'true' > "$__ush_return_path"
      elif [ "${__ush_capture_return:-0}" = '1' ]; then
        printf '%s' 'true' >&4
      fi
      return 0
      ;;
    *)
      __ush_at 9 '_ => false'
      if [ -n "${__ush_return_path:-}" ]; then
        printf '%s' 'false' > "$__ush_return_path"
      elif [ "${__ush_capture_return:-0}" = '1' ]; then
        printf '%s' 'false' >&4
      fi
      return 0
      ;;
  esac
}

__ush_at 13 'let expected = $env::VIZE_BASELINE_EXPECTED_SOURCE'
__ush_env_0=${VIZE_BASELINE_EXPECTED_SOURCE-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable VIZE_BASELINE_EXPECTED_SOURCE is not set or is empty'; exit 1; fi
__ush_binding_13_expected="${__ush_env_0}"
# shellcheck disable=SC2034
expected="${__ush_binding_13_expected}"
__ush_at 15 'if !(is_commit_sha $expected) {'
if ! {
__ush_value_1="$(__ush_return_path='' __ush_capture_return='1' ush_fn_is_commit_sha "${__ush_binding_13_expected}" 4>&1 >&3)" || exit "$?"
[ "${__ush_value_1}" = 'true' ]
}; then
  __ush_at 16 'fail 1 $"The dispatch source $expected is not a full commit SHA"'
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'The dispatch source '"${__ush_binding_13_expected}"' is not a full commit SHA'
  exit "$__ush_status"
fi
__ush_at 18 'let checkout = git rev-parse HEAD'
__ush_value_2="$(__ush_return_path='' __ush_capture_return='1' command git 'rev-parse' 'HEAD')" || exit "$?"
__ush_binding_18_checkout="${__ush_value_2}"
# shellcheck disable=SC2034
checkout="${__ush_binding_18_checkout}"
__ush_at 19 'if $checkout != $expected {'
if {
[ "$(if [ "${__ush_binding_18_checkout}" != "${__ush_binding_13_expected}" ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 20 'fail 1 $"The checkout is $checkout, not the dispatch source $expected"'
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'The checkout is '"${__ush_binding_18_checkout}"', not the dispatch source '"${__ush_binding_13_expected}"
  exit "$__ush_status"
fi
__ush_at 22 'if $env::GITHUB_SHA != $expected {'
if {
__ush_env_3=${GITHUB_SHA-}
if [ -z "${__ush_env_3}" ]; then __ush_report 1 'environment variable GITHUB_SHA is not set or is empty'; exit 1; fi
[ "$(if [ "${__ush_env_3}" != "${__ush_binding_13_expected}" ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 23 'fail 1 $"The run'"'"'s commit is $env::GITHUB_SHA, not the dispatch source $expected"'
  __ush_env_4=${GITHUB_SHA-}
  if [ -z "${__ush_env_4}" ]; then __ush_report 1 'environment variable GITHUB_SHA is not set or is empty'; exit 1; fi
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'The run'"'"'s commit is '"${__ush_env_4}"', not the dispatch source '"${__ush_binding_13_expected}"
  exit "$__ush_status"
fi
