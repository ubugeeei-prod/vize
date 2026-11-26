#!/bin/sh
# Generated from '../select-changed-product-inputs.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../select-changed-product-inputs.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../select-changed-product-inputs.ush' --checked-in -o 'select-changed-product-inputs.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'select-changed-product-inputs.ush'
__ush_origin='../select-changed-product-inputs.ush'

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
  -h|--help) printf '%s' 'select-changed-product-inputs.ush

Usage:
  select-changed-product-inputs.ush [--help]
  select-changed-product-inputs.ush [--man [ITEM]]
  select-changed-product-inputs.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
select-changed-product-inputs.ush

SYNOPSIS
select-changed-product-inputs.ush [--help]
select-changed-product-inputs.ush [--man [ITEM]]
select-changed-product-inputs.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

exec 3>&1

__ush_std_env_validate() {
  case "$1" in
    ''|[0-9]*|*[!A-Za-z0-9_]*)
      printf '%s\n' "ush std::env: invalid env name: $1" >&2
      return 1
      ;;
  esac
}

ush_fn_std__env__get() {
  __ush_std_env_validate "$1" || return "$?"
  eval "printf '%s' \"\${$1-}\""
}

ush_fn_is_base_commit() {
  __ush_fn_is_base_commit_arg_0="${1}"
  # shellcheck disable=SC2034
  sha="${__ush_fn_is_base_commit_arg_0}"
  __ush_at 8 'match sha {'
  case "${__ush_fn_is_base_commit_arg_0}" in
    *[!0123456789abcdef]*)
      __ush_at 9 'glob "*[!0-9a-f]*" => false'
      if [ -n "${__ush_return_path:-}" ]; then
        printf '%s' 'false' > "$__ush_return_path"
      elif [ "${__ush_capture_return:-0}" = '1' ]; then
        printf '%s' 'false' >&4
      fi
      return 0
      ;;
    '0000000000000000000000000000000000000000')
      __ush_at 10 '"0000000000000000000000000000000000000000" => false'
      if [ -n "${__ush_return_path:-}" ]; then
        printf '%s' 'false' > "$__ush_return_path"
      elif [ "${__ush_capture_return:-0}" = '1' ]; then
        printf '%s' 'false' >&4
      fi
      return 0
      ;;
    ????????????????????????????????????????)
      __ush_at 11 'glob "????????????????????????????????????????" => true'
      if [ -n "${__ush_return_path:-}" ]; then
        printf '%s' 'true' > "$__ush_return_path"
      elif [ "${__ush_capture_return:-0}" = '1' ]; then
        printf '%s' 'true' >&4
      fi
      return 0
      ;;
    *)
      __ush_at 12 '_ => false'
      if [ -n "${__ush_return_path:-}" ]; then
        printf '%s' 'false' > "$__ush_return_path"
      elif [ "${__ush_capture_return:-0}" = '1' ]; then
        printf '%s' 'false' >&4
      fi
      return 0
      ;;
  esac
}

__ush_at 17 'let base_sha = get BASE_SHA'
__ush_value_0="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__env__get 'BASE_SHA')" || exit "$?"
__ush_binding_17_base_sha="${__ush_value_0}"
# shellcheck disable=SC2034
base_sha="${__ush_binding_17_base_sha}"
__ush_at 19 'if is_base_commit $base_sha {'
if {
__ush_value_1="$(__ush_return_path='' __ush_capture_return='1' ush_fn_is_base_commit "${__ush_binding_17_base_sha}" 4>&1 >&3)" || exit "$?"
[ "${__ush_value_1}" = 'true' ]
}; then
  __ush_at 20 'git fetch --no-tags --depth=1 origin $base_sha'
  command git 'fetch' '--no-tags' '--depth=1' 'origin' "${__ush_binding_17_base_sha}"
fi
__ush_at 22 'node tools/support/compat/github/plan-product-integrations.ts \'
__ush_env_2=${GITHUB_EVENT_NAME-}
if [ -z "${__ush_env_2}" ]; then __ush_report 1 'environment variable GITHUB_EVENT_NAME is not set or is empty'; exit 1; fi
__ush_env_3=${GITHUB_SHA-}
if [ -z "${__ush_env_3}" ]; then __ush_report 1 'environment variable GITHUB_SHA is not set or is empty'; exit 1; fi
command node 'tools/support/compat/github/plan-product-integrations.ts' "${__ush_env_2}" "${__ush_binding_17_base_sha}" "${__ush_env_3}"
