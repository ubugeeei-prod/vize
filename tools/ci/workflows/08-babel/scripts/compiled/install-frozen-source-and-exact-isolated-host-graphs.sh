#!/bin/sh
# Generated from '../install-frozen-source-and-exact-isolated-host-graphs.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../install-frozen-source-and-exact-isolated-host-graphs.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../install-frozen-source-and-exact-isolated-host-graphs.ush' --checked-in -o 'install-frozen-source-and-exact-isolated-host-graphs.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'install-frozen-source-and-exact-isolated-host-graphs.ush'
__ush_origin='../install-frozen-source-and-exact-isolated-host-graphs.ush'

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
  -h|--help) printf '%s' 'install-frozen-source-and-exact-isolated-host-graphs.ush

Usage:
  install-frozen-source-and-exact-isolated-host-graphs.ush [--help]
  install-frozen-source-and-exact-isolated-host-graphs.ush [--man [ITEM]]
  install-frozen-source-and-exact-isolated-host-graphs.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
install-frozen-source-and-exact-isolated-host-graphs.ush

SYNOPSIS
install-frozen-source-and-exact-isolated-host-graphs.ush [--help]
install-frozen-source-and-exact-isolated-host-graphs.ush [--man [ITEM]]
install-frozen-source-and-exact-isolated-host-graphs.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_at 4 'let fixtures = "npm/builder/babel/test"'
__ush_binding_4_fixtures='npm/builder/babel/test'
# shellcheck disable=SC2034
fixtures="${__ush_binding_4_fixtures}"
__ush_at 6 'node tools/support/github/vp-install.ts --filter ./npm/builder/babel... --filter ./npm/native...'
command node 'tools/support/github/vp-install.ts' '--filter' './npm/builder/babel...' '--filter' './npm/native...'
__ush_at 7 'for host in ["babel7", "babel8"] {'
__ush_match_0__len='2'
# shellcheck disable=SC2034
__ush_match_0__0='babel7'
# shellcheck disable=SC2034
__ush_match_0__1='babel8'
__ush_for_list_index_1=0
while [ "${__ush_for_list_index_1}" -lt "${__ush_match_0__len}" ]; do
  __ush_for_value_2=''
  eval "__ush_for_value_2=\"\${__ush_match_0__${__ush_for_list_index_1}}\""
  host="${__ush_for_value_2}"
  __ush_for_list_index_1=$((__ush_for_list_index_1 + 1))
  __ush_at 8 'npm ci --prefix $fixtures/hosts/fixtures/$host --ignore-scripts --no-audit --no-fund'
  __ush_value_3="${__ush_binding_4_fixtures}"
  __ush_value_4="${host}"
  command npm 'ci' '--prefix' "${__ush_value_3}"'/hosts/fixtures/'"${__ush_value_4}" '--ignore-scripts' '--no-audit' '--no-fund'
done
__ush_at 10 'for runtime in ["vdom", "vapor-rc9", "vapor-rc11"] {'
__ush_match_5__len='3'
# shellcheck disable=SC2034
__ush_match_5__0='vdom'
# shellcheck disable=SC2034
__ush_match_5__1='vapor-rc9'
# shellcheck disable=SC2034
__ush_match_5__2='vapor-rc11'
__ush_for_list_index_6=0
while [ "${__ush_for_list_index_6}" -lt "${__ush_match_5__len}" ]; do
  __ush_for_value_7=''
  eval "__ush_for_value_7=\"\${__ush_match_5__${__ush_for_list_index_6}}\""
  runtime="${__ush_for_value_7}"
  __ush_for_list_index_6=$((__ush_for_list_index_6 + 1))
  __ush_at 11 'npm ci --prefix $fixtures/runtime/fixtures/$runtime --ignore-scripts --no-audit --no-fund'
  __ush_value_8="${__ush_binding_4_fixtures}"
  __ush_value_9="${runtime}"
  command npm 'ci' '--prefix' "${__ush_value_8}"'/runtime/fixtures/'"${__ush_value_9}" '--ignore-scripts' '--no-audit' '--no-fund'
done
__ush_at 13 'node $fixtures/runtime/fixtures/vdom/node_modules/playwright/cli.js install --with-deps chromium'
__ush_value_10="${__ush_binding_4_fixtures}"
command node "${__ush_value_10}"'/runtime/fixtures/vdom/node_modules/playwright/cli.js' 'install' '--with-deps' 'chromium'
