#!/bin/sh
# Generated from '../special-text-compare-the-original-packet-with-native-and-legacy-rust.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../special-text-compare-the-original-packet-with-native-and-legacy-rust.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../special-text-compare-the-original-packet-with-native-and-legacy-rust.ush' --checked-in -o 'special-text-compare-the-original-packet-with-native-and-legacy-rust.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'special-text-compare-the-original-packet-with-native-and-legacy-rust.ush'
__ush_origin='../special-text-compare-the-original-packet-with-native-and-legacy-rust.ush'

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
  -h|--help) printf '%s' 'special-text-compare-the-original-packet-with-native-and-legacy-rust.ush

Usage:
  special-text-compare-the-original-packet-with-native-and-legacy-rust.ush [--help]
  special-text-compare-the-original-packet-with-native-and-legacy-rust.ush [--man [ITEM]]
  special-text-compare-the-original-packet-with-native-and-legacy-rust.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
special-text-compare-the-original-packet-with-native-and-legacy-rust.ush

SYNOPSIS
special-text-compare-the-original-packet-with-native-and-legacy-rust.ush [--help]
special-text-compare-the-original-packet-with-native-and-legacy-rust.ush [--man [ITEM]]
special-text-compare-the-original-packet-with-native-and-legacy-rust.ush [--complete [PREFIX]]
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
__ush_at 8 'cargo test --locked --profile ci -p vize_l1 \'
command cargo 'test' '--locked' '--profile' 'ci' '-p' 'vize_l1' '--test' 'html_body_elements' '--test' 'html_body_elements_legacy_oracle' '--test' 'html_disclosure_elements' '--test' 'html_disclosure_elements_legacy_oracle' '--test' 'html_embedded_fallback' '--test' 'html_embedded_fallback_legacy_oracle' '--test' 'html_foreign_content' '--test' 'html_foreign_content_legacy_oracle' '--test' 'html_phrasing' '--test' 'html_phrasing_legacy_oracle' '--test' 'html_select' '--test' 'html_select_legacy_oracle' '--test' 'html_special_text' '--test' 'html_special_text_legacy_oracle'
