#!/bin/sh
# Generated from '../install-japanese-rendering-fonts.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../install-japanese-rendering-fonts.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../install-japanese-rendering-fonts.ush' --checked-in -o 'install-japanese-rendering-fonts.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'install-japanese-rendering-fonts.ush'
__ush_origin='../install-japanese-rendering-fonts.ush'

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
  -h|--help) printf '%s' 'install-japanese-rendering-fonts.ush

Usage:
  install-japanese-rendering-fonts.ush [--help]
  install-japanese-rendering-fonts.ush [--man [ITEM]]
  install-japanese-rendering-fonts.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
install-japanese-rendering-fonts.ush

SYNOPSIS
install-japanese-rendering-fonts.ush [--help]
install-japanese-rendering-fonts.ush [--man [ITEM]]
install-japanese-rendering-fonts.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_at 4 'sudo apt-get update'
command sudo 'apt-get' 'update'
__ush_at 5 'sudo apt-get install -y --no-install-recommends fonts-noto-cjk'
command sudo 'apt-get' 'install' '-y' '--no-install-recommends' 'fonts-noto-cjk'
__ush_at 6 'fc_match --format "%{family}\n" :lang=ja'
command fc-match '--format' '%{family}'"${__ush_lf}"'' ':lang=ja'
