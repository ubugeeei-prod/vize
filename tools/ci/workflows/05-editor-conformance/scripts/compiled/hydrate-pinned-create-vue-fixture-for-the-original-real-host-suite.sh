#!/bin/sh
# Generated from '../hydrate-pinned-create-vue-fixture-for-the-original-real-host-suite.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../hydrate-pinned-create-vue-fixture-for-the-original-real-host-suite.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../hydrate-pinned-create-vue-fixture-for-the-original-real-host-suite.ush' --checked-in -o 'hydrate-pinned-create-vue-fixture-for-the-original-real-host-suite.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'hydrate-pinned-create-vue-fixture-for-the-original-real-host-suite.ush'
__ush_origin='../hydrate-pinned-create-vue-fixture-for-the-original-real-host-suite.ush'

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
  -h|--help) printf '%s' 'hydrate-pinned-create-vue-fixture-for-the-original-real-host-suite.ush

Usage:
  hydrate-pinned-create-vue-fixture-for-the-original-real-host-suite.ush [--help]
  hydrate-pinned-create-vue-fixture-for-the-original-real-host-suite.ush [--man [ITEM]]
  hydrate-pinned-create-vue-fixture-for-the-original-real-host-suite.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
hydrate-pinned-create-vue-fixture-for-the-original-real-host-suite.ush

SYNOPSIS
hydrate-pinned-create-vue-fixture-for-the-original-real-host-suite.ush [--help]
hydrate-pinned-create-vue-fixture-for-the-original-real-host-suite.ush [--man [ITEM]]
hydrate-pinned-create-vue-fixture-for-the-original-real-host-suite.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_at 4 'let fixture = "tests/_fixtures/_git/create-vue"'
__ush_binding_4_fixture='tests/_fixtures/_git/create-vue'
# shellcheck disable=SC2034
fixture="${__ush_binding_4_fixture}"
__ush_at 6 'git submodule update --init --force --depth 1 -- $fixture'
command git 'submodule' 'update' '--init' '--force' '--depth' 1 '--' "${__ush_binding_4_fixture}"
__ush_at 7 'if (git rev-parse $"HEAD:$fixture") != (git -C $fixture rev-parse HEAD) {'
if {
__ush_value_0="$(__ush_return_path='' __ush_capture_return='1' command git 'rev-parse' 'HEAD:'"${__ush_binding_4_fixture}")" || exit "$?"
__ush_value_1="$(__ush_return_path='' __ush_capture_return='1' command git '-C' "${__ush_binding_4_fixture}" 'rev-parse' 'HEAD')" || exit "$?"
[ "$(if [ "${__ush_value_0}" != "${__ush_value_1}" ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 8 'fail 1 $"$fixture is not at the commit this revision pins"'
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" "${__ush_binding_4_fixture}"' is not at the commit this revision pins'
  exit "$__ush_status"
fi
