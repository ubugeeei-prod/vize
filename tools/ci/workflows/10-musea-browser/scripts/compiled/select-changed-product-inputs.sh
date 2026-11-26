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

ush_fn_std__regex__is_match() {
  VALUE="$1" PATTERN="$2" awk 'BEGIN { value = ENVIRON["VALUE"]; pattern = ENVIRON["PATTERN"]; if (value ~ pattern) { printf "true" } else { printf "false" } }'
}

__ush_at 6 'let base_sha = get BASE_SHA'
__ush_value_0="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__env__get 'BASE_SHA')" || exit "$?"
__ush_binding_6_base_sha="${__ush_value_0}"
# shellcheck disable=SC2034
base_sha="${__ush_binding_6_base_sha}"
__ush_at 9 'if is_match $base_sha "^[a-f0-9]{40}$" && !(is_match $base_sha "^0+$") {'
if {
{
__ush_value_1="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__regex__is_match "${__ush_binding_6_base_sha}" '^[a-f0-9]{40}$')" || exit "$?"
[ "${__ush_value_1}" = 'true' ]
} && ! {
__ush_value_2="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__regex__is_match "${__ush_binding_6_base_sha}" '^0+$')" || exit "$?"
[ "${__ush_value_2}" = 'true' ]
}
}; then
  __ush_at 10 'git fetch --no-tags --depth=1 origin $base_sha'
  command git 'fetch' '--no-tags' '--depth=1' 'origin' "${__ush_binding_6_base_sha}"
fi
__ush_at 12 'node tools/support/compat/github/plan-product-integrations.ts $env::GITHUB_EVENT_NAME $base_sha $env::GITHUB_SHA'
__ush_env_3=${GITHUB_EVENT_NAME-}
if [ -z "${__ush_env_3}" ]; then __ush_report 1 'environment variable GITHUB_EVENT_NAME is not set or is empty'; exit 1; fi
__ush_env_4=${GITHUB_SHA-}
if [ -z "${__ush_env_4}" ]; then __ush_report 1 'environment variable GITHUB_SHA is not set or is empty'; exit 1; fi
command node 'tools/support/compat/github/plan-product-integrations.ts' "${__ush_env_3}" "${__ush_binding_6_base_sha}" "${__ush_env_4}"
