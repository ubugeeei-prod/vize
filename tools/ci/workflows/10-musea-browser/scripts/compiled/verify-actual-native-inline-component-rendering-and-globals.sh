#!/bin/sh
# Generated from '../verify-actual-native-inline-component-rendering-and-globals.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../verify-actual-native-inline-component-rendering-and-globals.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../verify-actual-native-inline-component-rendering-and-globals.ush' --checked-in -o 'verify-actual-native-inline-component-rendering-and-globals.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'verify-actual-native-inline-component-rendering-and-globals.ush'
__ush_origin='../verify-actual-native-inline-component-rendering-and-globals.ush'

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
  -h|--help) printf '%s' 'verify-actual-native-inline-component-rendering-and-globals.ush

Usage:
  verify-actual-native-inline-component-rendering-and-globals.ush [--help]
  verify-actual-native-inline-component-rendering-and-globals.ush [--man [ITEM]]
  verify-actual-native-inline-component-rendering-and-globals.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
verify-actual-native-inline-component-rendering-and-globals.ush

SYNOPSIS
verify-actual-native-inline-component-rendering-and-globals.ush [--help]
verify-actual-native-inline-component-rendering-and-globals.ush [--man [ITEM]]
verify-actual-native-inline-component-rendering-and-globals.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_at 4 'vp exec tsx --test \'
command vp 'exec' 'tsx' '--test' 'src/plugin/inline-art-query.test.ts' 'src/plugin/art-processing.test.ts' 'src/inline-art-query.browser.test.ts'
__ush_at 8 'vp exec tsx --test \'
command vp 'exec' 'tsx' '--test' 'src/runtime-props-native.browser.test.ts' 'src/runtime-props-quoted-native.test.ts' 'src/optional-prop-spacing-native.test.ts' 'src/readonly-primitive-native.test.ts'
__ush_at 13 'vp exec tsx --test \'
command vp 'exec' 'tsx' '--test' 'src/art-component.test.ts' 'src/inline-art.browser.test.ts' 'src/vrt-api.browser.test.ts' 'src/vrt-report.browser.test.ts' 'src/vrt/report-identity.test.ts' 'src/usage-component.browser.test.ts' 'src/compact-props.test.ts' 'src/api-routes/handler-palette-compact.test.ts' 'src/compact-props.browser.test.ts' 'src/cli/serve.test.ts' 'src/cli/service/input.test.ts' 'src/cli/service/reports.test.ts' 'src/hosted-vrt-service.browser.test.ts' 'src/hosted-vrt-lifecycle.browser.test.ts' 'src/literal-variant-keys.browser.test.ts' 'src/literal-variant-keys.test.ts' 'src/hosted-vrt-literal.browser.test.ts' 'src/hosted-vrt-reconnect.browser.test.ts' 'src/packed-gallery-base.browser.test.ts' 'src/unsupported-props.browser.test.ts' 'gallery/utils/supportedProps.test.ts'
__ush_at 35 'vp exec tsx --test src/inline-art-static.browser.test.ts'
command vp 'exec' 'tsx' '--test' 'src/inline-art-static.browser.test.ts'
__ush_at 36 'vp exec tsx --test \'
command vp 'exec' 'tsx' '--test' 'src/variant-bindings-native.browser.test.ts' 'src/variant-bindings-http.test.ts'
__ush_at 39 'vp exec tsx --test \'
command vp 'exec' 'tsx' '--test' 'src/variant-navigation-owner.test.ts' 'src/variant-navigation-probe.browser.test.ts' 'src/variant-navigation-owner.browser.test.ts'
__ush_at 43 'vp exec tsx --test src/vrt/before-capture.browser.test.ts'
command vp 'exec' 'tsx' '--test' 'src/vrt/before-capture.browser.test.ts'
