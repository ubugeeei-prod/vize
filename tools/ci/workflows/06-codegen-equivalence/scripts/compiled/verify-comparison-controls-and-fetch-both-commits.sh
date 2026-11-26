#!/bin/sh
# Generated from '../verify-comparison-controls-and-fetch-both-commits.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../verify-comparison-controls-and-fetch-both-commits.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../verify-comparison-controls-and-fetch-both-commits.ush' --checked-in -o 'verify-comparison-controls-and-fetch-both-commits.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'verify-comparison-controls-and-fetch-both-commits.ush'
__ush_origin='../verify-comparison-controls-and-fetch-both-commits.ush'

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
  -h|--help) printf '%s' 'verify-comparison-controls-and-fetch-both-commits.ush

Usage:
  verify-comparison-controls-and-fetch-both-commits.ush [--help]
  verify-comparison-controls-and-fetch-both-commits.ush [--man [ITEM]]
  verify-comparison-controls-and-fetch-both-commits.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
verify-comparison-controls-and-fetch-both-commits.ush

SYNOPSIS
verify-comparison-controls-and-fetch-both-commits.ush [--help]
verify-comparison-controls-and-fetch-both-commits.ush [--man [ITEM]]
verify-comparison-controls-and-fetch-both-commits.ush [--complete [PREFIX]]
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

__ush_at 13 'export VIZE_CODEGEN_PROBE = "1" {'
__ush_had_VIZE_CODEGEN_PROBE_0=${VIZE_CODEGEN_PROBE+x} __ush_saved_VIZE_CODEGEN_PROBE_0=${VIZE_CODEGEN_PROBE-}
export VIZE_CODEGEN_PROBE='1'
  __ush_at 14 'node --test tests/tooling/levels-codegen-equivalence.test.ts'
  command node '--test' 'tests/tooling/levels-codegen-equivalence.test.ts'
if [ -n "$__ush_had_VIZE_CODEGEN_PROBE_0" ]; then export VIZE_CODEGEN_PROBE="$__ush_saved_VIZE_CODEGEN_PROBE_0"; else unset VIZE_CODEGEN_PROBE; fi
__ush_at 16 'for sha in [$env::BASE_SHA, $env::HEAD_SHA] {'
__ush_match_1__len='2'
__ush_env_2=${BASE_SHA-}
if [ -z "${__ush_env_2}" ]; then __ush_report 1 'environment variable BASE_SHA is not set or is empty'; exit 1; fi
# shellcheck disable=SC2034
__ush_match_1__0="${__ush_env_2}"
__ush_env_3=${HEAD_SHA-}
if [ -z "${__ush_env_3}" ]; then __ush_report 1 'environment variable HEAD_SHA is not set or is empty'; exit 1; fi
# shellcheck disable=SC2034
__ush_match_1__1="${__ush_env_3}"
__ush_for_list_index_4=0
while [ "${__ush_for_list_index_4}" -lt "${__ush_match_1__len}" ]; do
  __ush_for_value_5=''
  eval "__ush_for_value_5=\"\${__ush_match_1__${__ush_for_list_index_4}}\""
  sha="${__ush_for_value_5}"
  __ush_for_list_index_4=$((__ush_for_list_index_4 + 1))
  __ush_at 17 'if !(is_commit_sha $sha) {'
  if ! {
  __ush_value_6="$(__ush_return_path='' __ush_capture_return='1' ush_fn_is_commit_sha "${sha}" 4>&1 >&3)" || exit "$?"
  [ "${__ush_value_6}" = 'true' ]
  }; then
    __ush_at 18 'fail 1 $"$sha is not a full commit SHA"'
    __ush_status=1
    if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
    __ush_report "$__ush_status" "${sha}"' is not a full commit SHA'
    exit "$__ush_status"
  fi
done
__ush_at 21 'git fetch --no-tags --depth=1 origin $env::BASE_SHA $env::HEAD_SHA'
__ush_env_7=${BASE_SHA-}
if [ -z "${__ush_env_7}" ]; then __ush_report 1 'environment variable BASE_SHA is not set or is empty'; exit 1; fi
__ush_env_8=${HEAD_SHA-}
if [ -z "${__ush_env_8}" ]; then __ush_report 1 'environment variable HEAD_SHA is not set or is empty'; exit 1; fi
command git 'fetch' '--no-tags' '--depth=1' 'origin' "${__ush_env_7}" "${__ush_env_8}"
