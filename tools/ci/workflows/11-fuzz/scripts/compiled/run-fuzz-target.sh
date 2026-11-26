#!/bin/sh
# Generated from '../run-fuzz-target.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../run-fuzz-target.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../run-fuzz-target.ush' --checked-in -o 'run-fuzz-target.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'run-fuzz-target.ush'
__ush_origin='../run-fuzz-target.ush'

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
  -h|--help) printf '%s' 'run-fuzz-target.ush

Usage:
  run-fuzz-target.ush [--help]
  run-fuzz-target.ush [--man [ITEM]]
  run-fuzz-target.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
run-fuzz-target.ush

SYNOPSIS
run-fuzz-target.ush [--help]
run-fuzz-target.ush [--man [ITEM]]
run-fuzz-target.ush [--complete [PREFIX]]
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

__ush_at 5 'let budget = match get FUZZ_MODE {'
__ush_value_1="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__env__get 'FUZZ_MODE')" || exit "$?"
__ush_match_0="${__ush_value_1}"
case "${__ush_match_0}" in
  'replay')
    __ush_at 7 '"replay" => "-runs=0"'
    __ush_binding_5_budget='-runs=0'
    ;;
  *)
    __ush_at 8 '_ => $"-max_total_time=$env::FUZZ_MAX_TOTAL_TIME"'
    __ush_env_2=${FUZZ_MAX_TOTAL_TIME-}
    if [ -z "${__ush_env_2}" ]; then __ush_report 1 'environment variable FUZZ_MAX_TOTAL_TIME is not set or is empty'; exit 1; fi
    __ush_binding_5_budget='-max_total_time='"${__ush_env_2}"
    ;;
esac
# shellcheck disable=SC2034
budget="${__ush_binding_5_budget}"
__ush_at 11 'cargo +nightly fuzz run --fuzz-dir tests/fuzz $env::FUZZ_TARGET -- \'
__ush_env_3=${FUZZ_TARGET-}
if [ -z "${__ush_env_3}" ]; then __ush_report 1 'environment variable FUZZ_TARGET is not set or is empty'; exit 1; fi
command cargo '+nightly' 'fuzz' 'run' '--fuzz-dir' 'tests/fuzz' "${__ush_env_3}" '--' "${__ush_binding_5_budget}" '-max_len=65536' '-timeout=25' '-rss_limit_mb=4096' '-malloc_limit_mb=2048'
