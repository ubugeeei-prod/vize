#!/bin/sh
# Generated from '../special-text-compare-the-original-packet-with-chromium.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../special-text-compare-the-original-packet-with-chromium.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../special-text-compare-the-original-packet-with-chromium.ush' --checked-in -o 'special-text-compare-the-original-packet-with-chromium.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'special-text-compare-the-original-packet-with-chromium.ush'
__ush_origin='../special-text-compare-the-original-packet-with-chromium.ush'

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
  -h|--help) printf '%s' 'special-text-compare-the-original-packet-with-chromium.ush

Usage:
  special-text-compare-the-original-packet-with-chromium.ush [--help]
  special-text-compare-the-original-packet-with-chromium.ush [--man [ITEM]]
  special-text-compare-the-original-packet-with-chromium.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
special-text-compare-the-original-packet-with-chromium.ush

SYNOPSIS
special-text-compare-the-original-packet-with-chromium.ush [--help]
special-text-compare-the-original-packet-with-chromium.ush [--man [ITEM]]
special-text-compare-the-original-packet-with-chromium.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_at 4 'vp node --test \'
command vp 'node' '--test' 'tests/tooling/davinci-document-body-elements.test.ts' 'tests/tooling/davinci-document-disclosure-elements.test.ts' 'tests/tooling/davinci-document-embedded-fallback.test.ts' 'tests/tooling/davinci-document-foreign-content.test.ts' 'tests/tooling/davinci-document-phrasing.test.ts' 'tests/tooling/davinci-document-select.test.ts' 'tests/tooling/davinci-document-special-text.test.ts'
