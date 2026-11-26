#!/bin/sh
# Generated from '../require-complete-source-io-diagnostic-success.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../require-complete-source-io-diagnostic-success.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../require-complete-source-io-diagnostic-success.ush' --checked-in -o 'require-complete-source-io-diagnostic-success.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'require-complete-source-io-diagnostic-success.ush'
__ush_origin='../require-complete-source-io-diagnostic-success.ush'

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
  -h|--help) printf '%s' 'require-complete-source-io-diagnostic-success.ush

Usage:
  require-complete-source-io-diagnostic-success.ush [--help]
  require-complete-source-io-diagnostic-success.ush [--man [ITEM]]
  require-complete-source-io-diagnostic-success.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
require-complete-source-io-diagnostic-success.ush

SYNOPSIS
require-complete-source-io-diagnostic-success.ush [--help]
require-complete-source-io-diagnostic-success.ush [--man [ITEM]]
require-complete-source-io-diagnostic-success.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_at 2 'fail 1 "Complete source IO diagnostic failed; retained raw evidence grants no gate waiver."'
__ush_status=1
if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
__ush_report "$__ush_status" 'Complete source IO diagnostic failed; retained raw evidence grants no gate waiver.'
exit "$__ush_status"
