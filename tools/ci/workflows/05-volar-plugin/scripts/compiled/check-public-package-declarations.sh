#!/bin/sh
# Generated from '../check-public-package-declarations.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../check-public-package-declarations.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../check-public-package-declarations.ush' --checked-in -o 'check-public-package-declarations.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'check-public-package-declarations.ush'
__ush_origin='../check-public-package-declarations.ush'

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
  -h|--help) printf '%s' 'check-public-package-declarations.ush

Usage:
  check-public-package-declarations.ush [--help]
  check-public-package-declarations.ush [--man [ITEM]]
  check-public-package-declarations.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
check-public-package-declarations.ush

SYNOPSIS
check-public-package-declarations.ush [--help]
check-public-package-declarations.ush [--man [ITEM]]
check-public-package-declarations.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_at 4 'node $"$env::RUNNER_TEMP/volar-plugin-deps/node_modules/typescript/bin/tsc" \'
__ush_env_0=${RUNNER_TEMP-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
command node "${__ush_env_0}"'/volar-plugin-deps/node_modules/typescript/bin/tsc' '--ignoreConfig' '--strict' '--noEmit' '--module' 'NodeNext' '--moduleResolution' 'NodeNext' '--target' 'ESNext' 'npm/volar/src/index.d.ts'
