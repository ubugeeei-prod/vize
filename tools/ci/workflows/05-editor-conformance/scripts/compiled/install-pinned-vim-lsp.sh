#!/bin/sh
# Generated from '../install-pinned-vim-lsp.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../install-pinned-vim-lsp.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../install-pinned-vim-lsp.ush' --checked-in -o 'install-pinned-vim-lsp.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'install-pinned-vim-lsp.ush'
__ush_origin='../install-pinned-vim-lsp.ush'

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
  -h|--help) printf '%s' 'install-pinned-vim-lsp.ush

Usage:
  install-pinned-vim-lsp.ush [--help]
  install-pinned-vim-lsp.ush [--man [ITEM]]
  install-pinned-vim-lsp.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
install-pinned-vim-lsp.ush

SYNOPSIS
install-pinned-vim-lsp.ush [--help]
install-pinned-vim-lsp.ush [--man [ITEM]]
install-pinned-vim-lsp.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_at 4 'let checkout = $"$env::RUNNER_TEMP/vim-lsp"'
__ush_env_0=${RUNNER_TEMP-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_4_checkout="${__ush_env_0}"'/vim-lsp'
# shellcheck disable=SC2034
checkout="${__ush_binding_4_checkout}"
__ush_at 6 'git init $checkout'
command git 'init' "${__ush_binding_4_checkout}"
__ush_at 7 'git -C $checkout remote add origin https://github.com/prabirshrestha/vim-lsp.git'
command git '-C' "${__ush_binding_4_checkout}" 'remote' 'add' 'origin' 'https://github.com/prabirshrestha/vim-lsp.git'
__ush_at 8 'git -C $checkout fetch --depth 1 origin $env::VIM_LSP_COMMIT'
__ush_env_1=${VIM_LSP_COMMIT-}
if [ -z "${__ush_env_1}" ]; then __ush_report 1 'environment variable VIM_LSP_COMMIT is not set or is empty'; exit 1; fi
command git '-C' "${__ush_binding_4_checkout}" 'fetch' '--depth' 1 'origin' "${__ush_env_1}"
__ush_at 9 'git -C $checkout checkout --detach FETCH_HEAD'
command git '-C' "${__ush_binding_4_checkout}" 'checkout' '--detach' 'FETCH_HEAD'
__ush_at 10 'if (git -C $checkout rev-parse HEAD) != $env::VIM_LSP_COMMIT {'
if {
__ush_value_2="$(__ush_return_path='' __ush_capture_return='1' command git '-C' "${__ush_binding_4_checkout}" 'rev-parse' 'HEAD')" || exit "$?"
__ush_env_3=${VIM_LSP_COMMIT-}
if [ -z "${__ush_env_3}" ]; then __ush_report 1 'environment variable VIM_LSP_COMMIT is not set or is empty'; exit 1; fi
[ "$(if [ "${__ush_value_2}" != "${__ush_env_3}" ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 11 'fail 1 $"vim-lsp is not at the pinned commit $env::VIM_LSP_COMMIT"'
  __ush_env_4=${VIM_LSP_COMMIT-}
  if [ -z "${__ush_env_4}" ]; then __ush_report 1 'environment variable VIM_LSP_COMMIT is not set or is empty'; exit 1; fi
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'vim-lsp is not at the pinned commit '"${__ush_env_4}"
  exit "$__ush_status"
fi
