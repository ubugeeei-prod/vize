#!/bin/sh
# Generated from '../scoped-nesting-compile-current-source-and-retain-all-prior-scoped-css-oracles.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../scoped-nesting-compile-current-source-and-retain-all-prior-scoped-css-oracles.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../scoped-nesting-compile-current-source-and-retain-all-prior-scoped-css-oracles.ush' --checked-in -o 'scoped-nesting-compile-current-source-and-retain-all-prior-scoped-css-oracles.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'scoped-nesting-compile-current-source-and-retain-all-prior-scoped-css-oracles.ush'
__ush_origin='../scoped-nesting-compile-current-source-and-retain-all-prior-scoped-css-oracles.ush'

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
  -h|--help) printf '%s' 'scoped-nesting-compile-current-source-and-retain-all-prior-scoped-css-oracles.ush

Usage:
  scoped-nesting-compile-current-source-and-retain-all-prior-scoped-css-oracles.ush [--help]
  scoped-nesting-compile-current-source-and-retain-all-prior-scoped-css-oracles.ush [--man [ITEM]]
  scoped-nesting-compile-current-source-and-retain-all-prior-scoped-css-oracles.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
scoped-nesting-compile-current-source-and-retain-all-prior-scoped-css-oracles.ush

SYNOPSIS
scoped-nesting-compile-current-source-and-retain-all-prior-scoped-css-oracles.ush [--help]
scoped-nesting-compile-current-source-and-retain-all-prior-scoped-css-oracles.ush [--man [ITEM]]
scoped-nesting-compile-current-source-and-retain-all-prior-scoped-css-oracles.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_at 4 'let checkout = git rev-parse HEAD'
__ush_value_0="$(__ush_return_path='' __ush_capture_return='1' command git 'rev-parse' 'HEAD')" || exit "$?"
__ush_binding_4_checkout="${__ush_value_0}"
# shellcheck disable=SC2034
checkout="${__ush_binding_4_checkout}"
__ush_at 5 'if $checkout != $env::HEAD_SHA {'
if {
__ush_env_1=${HEAD_SHA-}
if [ -z "${__ush_env_1}" ]; then __ush_report 1 'environment variable HEAD_SHA is not set or is empty'; exit 1; fi
[ "$(if [ "${__ush_binding_4_checkout}" != "${__ush_env_1}" ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 6 'fail 1 $"The checkout is $checkout, not the commit under test $env::HEAD_SHA"'
  __ush_env_2=${HEAD_SHA-}
  if [ -z "${__ush_env_2}" ]; then __ush_report 1 'environment variable HEAD_SHA is not set or is empty'; exit 1; fi
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'The checkout is '"${__ush_binding_4_checkout}"', not the commit under test '"${__ush_env_2}"
  exit "$__ush_status"
fi
__ush_at 8 'cargo test --locked --profile ci -p vize_atelier_sfc --lib vite_plugin::css_scope:: -- --nocapture'
command cargo 'test' '--locked' '--profile' 'ci' '-p' 'vize_atelier_sfc' '--lib' 'vite_plugin::css_scope::' '--' '--nocapture'
__ush_at 9 'cargo test --locked --profile ci -p vize_atelier_sfc --test scoped_nesting_specificity -- --nocapture'
command cargo 'test' '--locked' '--profile' 'ci' '-p' 'vize_atelier_sfc' '--test' 'scoped_nesting_specificity' '--' '--nocapture'
__ush_at 10 'cargo test --locked --profile ci -p vize_atelier_sfc --test mixed_style_scope -- --nocapture'
command cargo 'test' '--locked' '--profile' 'ci' '-p' 'vize_atelier_sfc' '--test' 'mixed_style_scope' '--' '--nocapture'
