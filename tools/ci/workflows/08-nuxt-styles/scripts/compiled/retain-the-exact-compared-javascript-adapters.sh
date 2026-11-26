#!/bin/sh
# Generated from '../retain-the-exact-compared-javascript-adapters.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../retain-the-exact-compared-javascript-adapters.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../retain-the-exact-compared-javascript-adapters.ush' --checked-in -o 'retain-the-exact-compared-javascript-adapters.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'retain-the-exact-compared-javascript-adapters.ush'
__ush_origin='../retain-the-exact-compared-javascript-adapters.ush'

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
  -h|--help) printf '%s' 'retain-the-exact-compared-javascript-adapters.ush

Usage:
  retain-the-exact-compared-javascript-adapters.ush [--help]
  retain-the-exact-compared-javascript-adapters.ush [--man [ITEM]]
  retain-the-exact-compared-javascript-adapters.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
retain-the-exact-compared-javascript-adapters.ush

SYNOPSIS
retain-the-exact-compared-javascript-adapters.ush [--help]
retain-the-exact-compared-javascript-adapters.ush [--man [ITEM]]
retain-the-exact-compared-javascript-adapters.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

ush_fn_std__fs__is_dir() {
  __ush_path=$(ush_fn_std__path__resolve "$1")
  if [ -d "$__ush_path" ]; then printf '%s' 'true'; else printf '%s' 'false'; fi
}

ush_fn_std__fs__mkdir_p() {
  __ush_path=$(ush_fn_std__path__resolve "$1")
  mkdir -p "$__ush_path"
}

__ush_std_path_join_parts() {
  __ush_left=$1
  __ush_right=$2
  if [ -z "$__ush_left" ]; then
    printf '%s' "$__ush_right"
    return 0
  fi
  if [ -z "$__ush_right" ]; then
    printf '%s' "$__ush_left"
    return 0
  fi
  while [ "$__ush_left" != "/" ] && [ "${__ush_left%/}" != "$__ush_left" ]; do
    __ush_left=${__ush_left%/}
  done
  while [ "${__ush_right#/}" != "$__ush_right" ]; do
    __ush_right=${__ush_right#/}
  done
  case "$__ush_left" in
    /) printf '/%s' "$__ush_right" ;;
    *) printf '%s/%s' "$__ush_left" "$__ush_right" ;;
  esac
}

__ush_std_path_resolve() {
  __ush_value=$1
  case "$__ush_value" in
    __ush_pathref__:cwd:*)
      __ush_std_path_join_parts "$(pwd)" "${__ush_value#__ush_pathref__:cwd:}"
      ;;
    __ush_pathref__:source:*)
      __ush_base=${__ush_source_dir:-}
      if [ -z "$__ush_base" ]; then
        __ush_base=$(pwd)
      fi
      __ush_std_path_join_parts "$__ush_base" "${__ush_value#__ush_pathref__:source:}"
      ;;
    *) printf '%s' "$__ush_value" ;;
  esac
}

ush_fn_std__path__resolve() {
  __ush_std_path_resolve "$1"
}

__ush_at 5 'let adapters = $"$env::RUNNER_TEMP/$env::PROOF/compared-adapters"'
__ush_env_0=${RUNNER_TEMP-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_env_1=${PROOF-}
if [ -z "${__ush_env_1}" ]; then __ush_report 1 'environment variable PROOF is not set or is empty'; exit 1; fi
__ush_binding_5_adapters="${__ush_env_0}"'/'"${__ush_env_1}"'/compared-adapters'
# shellcheck disable=SC2034
adapters="${__ush_binding_5_adapters}"
__ush_at 7 'mkdir_p $adapters/candidate'
__ush_value_2="${__ush_binding_5_adapters}"
( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__mkdir_p "${__ush_value_2}"'/candidate' )
__ush_at 8 'mkdir_p $adapters/published'
__ush_value_3="${__ush_binding_5_adapters}"
( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__mkdir_p "${__ush_value_3}"'/published' )
__ush_at 9 'for adapter in ["vite-plugin", "nuxt"] {'
__ush_match_4__len='2'
# shellcheck disable=SC2034
__ush_match_4__0='vite-plugin'
# shellcheck disable=SC2034
__ush_match_4__1='nuxt'
__ush_for_list_index_5=0
while [ "${__ush_for_list_index_5}" -lt "${__ush_match_4__len}" ]; do
  __ush_for_value_6=''
  eval "__ush_for_value_6=\"\${__ush_match_4__${__ush_for_list_index_5}}\""
  adapter="${__ush_for_value_6}"
  __ush_for_list_index_5=$((__ush_for_list_index_5 + 1))
  __ush_at 10 'let published = $"$env::FIXTURE/node_modules/@vizejs/$adapter/dist"'
  __ush_env_7=${FIXTURE-}
  if [ -z "${__ush_env_7}" ]; then __ush_report 1 'environment variable FIXTURE is not set or is empty'; exit 1; fi
  __ush_binding_10_published="${__ush_env_7}"'/node_modules/@vizejs/'"${adapter}"'/dist'
  # shellcheck disable=SC2034
  published="${__ush_binding_10_published}"
  __ush_at 11 'if is_dir $published {'
  if {
  __ush_value_8="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__fs__is_dir "${__ush_binding_10_published}")" || exit "$?"
  [ "${__ush_value_8}" = 'true' ]
  }; then
    __ush_at 12 'cp -R $published $adapters/published/$adapter'
    __ush_value_9="${__ush_binding_5_adapters}"
    __ush_value_10="${adapter}"
    command cp '-R' "${__ush_binding_10_published}" "${__ush_value_9}"'/published/'"${__ush_value_10}"
  fi
done
__ush_at 15 'if is_dir "npm/builder/vite/dist" {'
if {
__ush_value_11="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__fs__is_dir 'npm/builder/vite/dist')" || exit "$?"
[ "${__ush_value_11}" = 'true' ]
}; then
  __ush_at 16 'cp -R npm/builder/vite/dist $adapters/candidate/vite-plugin'
  __ush_value_12="${__ush_binding_5_adapters}"
  command cp '-R' 'npm/builder/vite/dist' "${__ush_value_12}"'/candidate/vite-plugin'
fi
__ush_at 18 'if is_dir "npm/framework/nuxt/dist" {'
if {
__ush_value_13="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__fs__is_dir 'npm/framework/nuxt/dist')" || exit "$?"
[ "${__ush_value_13}" = 'true' ]
}; then
  __ush_at 19 'cp -R npm/framework/nuxt/dist $adapters/candidate/nuxt'
  __ush_value_14="${__ush_binding_5_adapters}"
  command cp '-R' 'npm/framework/nuxt/dist' "${__ush_value_14}"'/candidate/nuxt'
fi
