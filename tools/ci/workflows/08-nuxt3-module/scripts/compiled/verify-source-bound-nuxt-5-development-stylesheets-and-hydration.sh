#!/bin/sh
# Generated from '../verify-source-bound-nuxt-5-development-stylesheets-and-hydration.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../verify-source-bound-nuxt-5-development-stylesheets-and-hydration.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../verify-source-bound-nuxt-5-development-stylesheets-and-hydration.ush' --checked-in -o 'verify-source-bound-nuxt-5-development-stylesheets-and-hydration.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'verify-source-bound-nuxt-5-development-stylesheets-and-hydration.ush'
__ush_origin='../verify-source-bound-nuxt-5-development-stylesheets-and-hydration.ush'

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
  -h|--help) printf '%s' 'verify-source-bound-nuxt-5-development-stylesheets-and-hydration.ush

Usage:
  verify-source-bound-nuxt-5-development-stylesheets-and-hydration.ush [--help]
  verify-source-bound-nuxt-5-development-stylesheets-and-hydration.ush [--man [ITEM]]
  verify-source-bound-nuxt-5-development-stylesheets-and-hydration.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
verify-source-bound-nuxt-5-development-stylesheets-and-hydration.ush

SYNOPSIS
verify-source-bound-nuxt-5-development-stylesheets-and-hydration.ush [--help]
verify-source-bound-nuxt-5-development-stylesheets-and-hydration.ush [--man [ITEM]]
verify-source-bound-nuxt-5-development-stylesheets-and-hydration.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

ush_fn_std__pack() {
  __ush_count=0
  eval "__ush_count=\${${1}__len}"
  __ush_i=0
  while [ "$__ush_i" -lt "$__ush_count" ]; do
    eval "__ush_element=\${${1}__${__ush_i}}"
    printf x
    printf '%s' "$__ush_element" | od -A n -v -t x1 | tr -d ' \n'
    printf ' '
    __ush_i=$((__ush_i + 1))
  done
}

ush_fn_std__unpack() {
  __ush_i=0
  for __ush_hex in $2; do
    __ush_element=$(LC_ALL=C awk -v s="${__ush_hex#x}" 'BEGIN {
      digits = "0123456789abcdef"
      for (i = 1; i <= length(s); i += 2)
        printf "%c", (index(digits, substr(s,i,1))-1)*16 + index(digits,substr(s,i+1,1))-1
      printf "."
    }')
    __ush_element=${__ush_element%.}
    eval "${1}__${__ush_i}=\$__ush_element"
    __ush_i=$((__ush_i + 1))
  done
  eval "${1}__len=\$__ush_i"
}

ush_fn_std__process__exec() {
  __ush_program=$1
  __ush_args__len=0
  __ush_arg=''
  ush_fn_std__unpack __ush_args "$2" || return "$?"
  set --
  __ush_index=0
  while [ "$__ush_index" -lt "$__ush_args__len" ]; do
    eval "__ush_arg=\${__ush_args__${__ush_index}}"
    set -- "$@" "$__ush_arg"
    __ush_index=$((__ush_index + 1))
  done
  "$__ush_program" "$@"
}

__ush_at 4 'let controller = "tools/support/compat/nuxt/nuxt5-dev-runtime-build.ts"'
__ush_binding_4_controller='tools/support/compat/nuxt/nuxt5-dev-runtime-build.ts'
# shellcheck disable=SC2034
controller="${__ush_binding_4_controller}"
__ush_at 5 'let acceptance = $"$env::RUNNER_TEMP/nuxt5-dev-runtime-proof/acceptance"'
__ush_env_0=${RUNNER_TEMP-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_5_acceptance="${__ush_env_0}"'/nuxt5-dev-runtime-proof/acceptance'
# shellcheck disable=SC2034
acceptance="${__ush_binding_5_acceptance}"
__ush_at 7 'let consumer_node = command -v node'
__ush_value_1="$(__ush_return_path='' __ush_capture_return='1' command command '-v' 'node')" || exit "$?"
__ush_binding_7_consumer_node="${__ush_value_1}"
# shellcheck disable=SC2034
consumer_node="${__ush_binding_7_consumer_node}"
__ush_at 10 'std::process::exec $env::VIZE_NUXT_SOURCE_NODE [controller, acceptance, consumer_node]'
__ush_env_2=${VIZE_NUXT_SOURCE_NODE-}
if [ -z "${__ush_env_2}" ]; then __ush_report 1 'environment variable VIZE_NUXT_SOURCE_NODE is not set or is empty'; exit 1; fi
__ush_match_3__len='3'
# shellcheck disable=SC2034
__ush_match_3__0="${__ush_binding_4_controller}"
# shellcheck disable=SC2034
__ush_match_3__1="${__ush_binding_5_acceptance}"
# shellcheck disable=SC2034
__ush_match_3__2="${__ush_binding_7_consumer_node}"
( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__process__exec "${__ush_env_2}" "$(ush_fn_std__pack __ush_match_3)" )
