#!/bin/sh
# Generated from '../src-ranges-prepare-the-exact-source-native-image.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../src-ranges-prepare-the-exact-source-native-image.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../src-ranges-prepare-the-exact-source-native-image.ush' --checked-in -o 'src-ranges-prepare-the-exact-source-native-image.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'src-ranges-prepare-the-exact-source-native-image.ush'
__ush_origin='../src-ranges-prepare-the-exact-source-native-image.ush'

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
  -h|--help) printf '%s' 'src-ranges-prepare-the-exact-source-native-image.ush

Usage:
  src-ranges-prepare-the-exact-source-native-image.ush [--help]
  src-ranges-prepare-the-exact-source-native-image.ush [--man [ITEM]]
  src-ranges-prepare-the-exact-source-native-image.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
src-ranges-prepare-the-exact-source-native-image.ush

SYNOPSIS
src-ranges-prepare-the-exact-source-native-image.ush [--help]
src-ranges-prepare-the-exact-source-native-image.ush [--man [ITEM]]
src-ranges-prepare-the-exact-source-native-image.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_at 4 'if (git rev-parse HEAD) != $env::SOURCE_SHA {'
if {
__ush_value_0="$(__ush_return_path='' __ush_capture_return='1' command git 'rev-parse' 'HEAD')" || exit "$?"
__ush_env_1=${SOURCE_SHA-}
if [ -z "${__ush_env_1}" ]; then __ush_report 1 'environment variable SOURCE_SHA is not set or is empty'; exit 1; fi
[ "$(if [ "${__ush_value_0}" != "${__ush_env_1}" ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 5 'fail 1 $"The checkout is not the source commit $env::SOURCE_SHA"'
  __ush_env_2=${SOURCE_SHA-}
  if [ -z "${__ush_env_2}" ]; then __ush_report 1 'environment variable SOURCE_SHA is not set or is empty'; exit 1; fi
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'The checkout is not the source commit '"${__ush_env_2}"
  exit "$__ush_status"
fi
__ush_at 7 'vp run --workspace-root build:native:test'
command vp 'run' '--workspace-root' 'build:native:test'
