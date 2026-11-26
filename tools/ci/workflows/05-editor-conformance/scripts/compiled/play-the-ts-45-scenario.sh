#!/bin/sh
# Generated from '../play-the-ts-45-scenario.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../play-the-ts-45-scenario.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../play-the-ts-45-scenario.ush' --checked-in -o 'play-the-ts-45-scenario.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'play-the-ts-45-scenario.ush'
__ush_origin='../play-the-ts-45-scenario.ush'

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
  -h|--help) printf '%s' 'play-the-ts-45-scenario.ush

Usage:
  play-the-ts-45-scenario.ush [--help]
  play-the-ts-45-scenario.ush [--man [ITEM]]
  play-the-ts-45-scenario.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
play-the-ts-45-scenario.ush

SYNOPSIS
play-the-ts-45-scenario.ush [--help]
play-the-ts-45-scenario.ush [--man [ITEM]]
play-the-ts-45-scenario.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_at 4 'let out = $"$env::RUNNER_TEMP/ts45"'
__ush_env_0=${RUNNER_TEMP-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_4_out="${__ush_env_0}"'/ts45'
# shellcheck disable=SC2034
out="${__ush_binding_4_out}"
__ush_at 6 'if $env::CLIENT == "vscode" {'
if {
__ush_env_1=${CLIENT-}
if [ -z "${__ush_env_1}" ]; then __ush_report 1 'environment variable CLIENT is not set or is empty'; exit 1; fi
[ "$(if [ "${__ush_env_1}" = 'vscode' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 7 'xvfb_run -a node tests/editor-conformance/run.ts $env::CLIENT --out $out'
  __ush_env_2=${CLIENT-}
  if [ -z "${__ush_env_2}" ]; then __ush_report 1 'environment variable CLIENT is not set or is empty'; exit 1; fi
  command xvfb-run '-a' 'node' 'tests/editor-conformance/run.ts' "${__ush_env_2}" '--out' "${__ush_binding_4_out}"
else
  __ush_at 9 'node tests/editor-conformance/run.ts $env::CLIENT --out $out'
  __ush_env_3=${CLIENT-}
  if [ -z "${__ush_env_3}" ]; then __ush_report 1 'environment variable CLIENT is not set or is empty'; exit 1; fi
  command node 'tests/editor-conformance/run.ts' "${__ush_env_3}" '--out' "${__ush_binding_4_out}"
fi
