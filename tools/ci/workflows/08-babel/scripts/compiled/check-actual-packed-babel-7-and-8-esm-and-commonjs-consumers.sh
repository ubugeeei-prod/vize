#!/bin/sh
# Generated from '../check-actual-packed-babel-7-and-8-esm-and-commonjs-consumers.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../check-actual-packed-babel-7-and-8-esm-and-commonjs-consumers.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../check-actual-packed-babel-7-and-8-esm-and-commonjs-consumers.ush' --checked-in -o 'check-actual-packed-babel-7-and-8-esm-and-commonjs-consumers.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'check-actual-packed-babel-7-and-8-esm-and-commonjs-consumers.ush'
__ush_origin='../check-actual-packed-babel-7-and-8-esm-and-commonjs-consumers.ush'

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
  -h|--help) printf '%s' 'check-actual-packed-babel-7-and-8-esm-and-commonjs-consumers.ush

Usage:
  check-actual-packed-babel-7-and-8-esm-and-commonjs-consumers.ush [--help]
  check-actual-packed-babel-7-and-8-esm-and-commonjs-consumers.ush [--man [ITEM]]
  check-actual-packed-babel-7-and-8-esm-and-commonjs-consumers.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
check-actual-packed-babel-7-and-8-esm-and-commonjs-consumers.ush

SYNOPSIS
check-actual-packed-babel-7-and-8-esm-and-commonjs-consumers.ush [--help]
check-actual-packed-babel-7-and-8-esm-and-commonjs-consumers.ush [--man [ITEM]]
check-actual-packed-babel-7-and-8-esm-and-commonjs-consumers.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_at 4 'let packed = $"$env::RUNNER_TEMP/babel-plugin/packed"'
__ush_env_0=${RUNNER_TEMP-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_4_packed="${__ush_env_0}"'/babel-plugin/packed'
# shellcheck disable=SC2034
packed="${__ush_binding_4_packed}"
__ush_at 5 'let packed_plugin = find $packed -maxdepth 1 -name *.tgz -print -quit'
__ush_value_1="$(__ush_return_path='' __ush_capture_return='1' command find "${__ush_binding_4_packed}" '-maxdepth' 1 '-name' '*.tgz' '-print' '-quit')" || exit "$?"
__ush_binding_5_packed_plugin="${__ush_value_1}"
# shellcheck disable=SC2034
packed_plugin="${__ush_binding_5_packed_plugin}"
__ush_at 7 'export VIZE_BABEL_PACKED_PLUGIN = $packed_plugin {'
__ush_had_VIZE_BABEL_PACKED_PLUGIN_2=${VIZE_BABEL_PACKED_PLUGIN+x} __ush_saved_VIZE_BABEL_PACKED_PLUGIN_2=${VIZE_BABEL_PACKED_PLUGIN-}
export VIZE_BABEL_PACKED_PLUGIN="${__ush_binding_5_packed_plugin}"
  __ush_at 8 'node npm/native/scripts/test-preparation.mjs node --test \'
  command node 'npm/native/scripts/test-preparation.mjs' 'node' '--test' 'npm/builder/babel/test/packed-hosts.test.ts'
if [ -n "$__ush_had_VIZE_BABEL_PACKED_PLUGIN_2" ]; then export VIZE_BABEL_PACKED_PLUGIN="$__ush_saved_VIZE_BABEL_PACKED_PLUGIN_2"; else unset VIZE_BABEL_PACKED_PLUGIN; fi
