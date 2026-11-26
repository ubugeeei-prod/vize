#!/bin/sh
# Generated from '../verify-real-accessibility-and-deterministic-screenshots.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../verify-real-accessibility-and-deterministic-screenshots.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../verify-real-accessibility-and-deterministic-screenshots.ush' --checked-in -o 'verify-real-accessibility-and-deterministic-screenshots.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'verify-real-accessibility-and-deterministic-screenshots.ush'
__ush_origin='../verify-real-accessibility-and-deterministic-screenshots.ush'

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
  -h|--help) printf '%s' 'verify-real-accessibility-and-deterministic-screenshots.ush

Usage:
  verify-real-accessibility-and-deterministic-screenshots.ush [--help]
  verify-real-accessibility-and-deterministic-screenshots.ush [--man [ITEM]]
  verify-real-accessibility-and-deterministic-screenshots.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
verify-real-accessibility-and-deterministic-screenshots.ush

SYNOPSIS
verify-real-accessibility-and-deterministic-screenshots.ush [--help]
verify-real-accessibility-and-deterministic-screenshots.ush [--man [ITEM]]
verify-real-accessibility-and-deterministic-screenshots.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_at 4 'vp exec tsx --test gallery/components/home-layout.browser.test.ts'
command vp 'exec' 'tsx' '--test' 'gallery/components/home-layout.browser.test.ts'
__ush_at 5 'vp exec tsx --test \'
command vp 'exec' 'tsx' '--test' 'src/a11y/browser-contracts.test.ts' 'src/a11y/node-report.test.ts' 'gallery/components/code-display.browser.test.ts' 'gallery/components/global-toolbar.browser.test.ts' 'gallery/components/public-browser-observation.browser.test.ts' 'gallery/components/hosted-tests.browser.test.ts' 'gallery/components/snapshot-collisions.browser.test.ts'
__ush_at 13 'vp exec tsx --test gallery/components/viewport-presets.browser.test.ts'
command vp 'exec' 'tsx' '--test' 'gallery/components/viewport-presets.browser.test.ts'
__ush_at 14 'vp exec tsx --test \'
command vp 'exec' 'tsx' '--test' 'src/props-permalink.test.ts' 'gallery/components/props-permalink.browser.test.ts'
__ush_at 17 'vp exec tsx --test gallery/components/sidebar-rotation.browser.test.ts'
command vp 'exec' 'tsx' '--test' 'gallery/components/sidebar-rotation.browser.test.ts'
__ush_at 18 'vp exec tsx --test src/vrt/mask-capture.browser.test.ts'
command vp 'exec' 'tsx' '--test' 'src/vrt/mask-capture.browser.test.ts'
__ush_at 19 'vp exec tsx --test src/vrt/diff-style.browser.test.ts'
command vp 'exec' 'tsx' '--test' 'src/vrt/diff-style.browser.test.ts'
__ush_at 20 'vp exec tsx --test src/vrt/before-capture.browser.test.ts'
command vp 'exec' 'tsx' '--test' 'src/vrt/before-capture.browser.test.ts'
__ush_at 21 'vp exec tsx --test gallery/components/content-layout.browser.test.ts'
command vp 'exec' 'tsx' '--test' 'gallery/components/content-layout.browser.test.ts'
