#!/bin/sh
# Generated from '../verify-rendered-navigation-and-japanese-reading-paths.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../verify-rendered-navigation-and-japanese-reading-paths.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../verify-rendered-navigation-and-japanese-reading-paths.ush' --checked-in -o 'verify-rendered-navigation-and-japanese-reading-paths.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'verify-rendered-navigation-and-japanese-reading-paths.ush'
__ush_origin='../verify-rendered-navigation-and-japanese-reading-paths.ush'

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
  -h|--help) printf '%s' 'verify-rendered-navigation-and-japanese-reading-paths.ush

Usage:
  verify-rendered-navigation-and-japanese-reading-paths.ush [--help]
  verify-rendered-navigation-and-japanese-reading-paths.ush [--man [ITEM]]
  verify-rendered-navigation-and-japanese-reading-paths.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
verify-rendered-navigation-and-japanese-reading-paths.ush

SYNOPSIS
verify-rendered-navigation-and-japanese-reading-paths.ush [--help]
verify-rendered-navigation-and-japanese-reading-paths.ush [--man [ITEM]]
verify-rendered-navigation-and-japanese-reading-paths.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_std_path_make_ref() {
  __ush_kind=$1
  __ush_value=$2
  case "$__ush_value" in
    ''|/*|__ush_pathref__:*) printf '%s' "$__ush_value" ;;
    *) printf '__ush_pathref__:%s:%s' "$__ush_kind" "$__ush_value" ;;
  esac
}

ush_fn_std__path__from_source() {
  __ush_std_path_make_ref source "$1"
}

__ush_at 5 'let ja_reading_routes = node (from_source "verify-rendered-navigation-and-japanese-reading-paths.ts")'
__ush_value_0="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__from_source 'verify-rendered-navigation-and-japanese-reading-paths.ts')" || exit "$?"
__ush_value_1="$(__ush_return_path='' __ush_capture_return='1' command node "${__ush_value_0}")" || exit "$?"
__ush_binding_5_ja_reading_routes="${__ush_value_1}"
# shellcheck disable=SC2034
ja_reading_routes="${__ush_binding_5_ja_reading_routes}"
__ush_at 7 'node docs/scripts/verify-navigation-render.ts --routes $ja_reading_routes'
command node 'docs/scripts/verify-navigation-render.ts' '--routes' "${__ush_binding_5_ja_reading_routes}"
__ush_at 8 'node docs/scripts/verify-ja-editorial-render.ts'
command node 'docs/scripts/verify-ja-editorial-render.ts'
