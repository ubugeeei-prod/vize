#!/bin/sh
# Generated from '../verify-original-nuxt-lint-configuration-with-the-source.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../verify-original-nuxt-lint-configuration-with-the-source.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../verify-original-nuxt-lint-configuration-with-the-source.ush' --checked-in -o 'verify-original-nuxt-lint-configuration-with-the-source.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'verify-original-nuxt-lint-configuration-with-the-source.ush'
__ush_origin='../verify-original-nuxt-lint-configuration-with-the-source.ush'

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
  -h|--help) printf '%s' 'verify-original-nuxt-lint-configuration-with-the-source.ush

Usage:
  verify-original-nuxt-lint-configuration-with-the-source.ush [--help]
  verify-original-nuxt-lint-configuration-with-the-source.ush [--man [ITEM]]
  verify-original-nuxt-lint-configuration-with-the-source.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
verify-original-nuxt-lint-configuration-with-the-source.ush

SYNOPSIS
verify-original-nuxt-lint-configuration-with-the-source.ush [--help]
verify-original-nuxt-lint-configuration-with-the-source.ush [--man [ITEM]]
verify-original-nuxt-lint-configuration-with-the-source.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_at 4 'vp env exec node tools/support/compat/nuxt/lint-config-build.ts \'
__ush_env_0=${RUNNER_TEMP-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_value_1="${__ush_env_0}"
__ush_env_2=${RUNNER_TEMP-}
if [ -z "${__ush_env_2}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_value_3="${__ush_env_2}"
command vp 'env' 'exec' 'node' 'tools/support/compat/nuxt/lint-config-build.ts' "${__ush_value_1}"'/nuxt-lint-config-proof' "${__ush_value_3}"'/nuxt3-module-proof/source-native/custody.json'
