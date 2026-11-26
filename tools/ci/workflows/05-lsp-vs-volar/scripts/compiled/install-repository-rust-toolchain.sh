#!/bin/sh
# Generated from '../install-repository-rust-toolchain.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../install-repository-rust-toolchain.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../install-repository-rust-toolchain.ush' --checked-in -o 'install-repository-rust-toolchain.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'install-repository-rust-toolchain.ush'
__ush_origin='../install-repository-rust-toolchain.ush'

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
  -h|--help) printf '%s' 'install-repository-rust-toolchain.ush

Usage:
  install-repository-rust-toolchain.ush [--help]
  install-repository-rust-toolchain.ush [--man [ITEM]]
  install-repository-rust-toolchain.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
install-repository-rust-toolchain.ush

SYNOPSIS
install-repository-rust-toolchain.ush [--help]
install-repository-rust-toolchain.ush [--man [ITEM]]
install-repository-rust-toolchain.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_at 5 'if ! rustup toolchain install {'
if ! {
command rustup 'toolchain' 'install'
}; then
  __ush_at 6 'rustup show active-toolchain'
  command rustup 'show' 'active-toolchain'
fi
__ush_at 8 'cargo --version'
command cargo '--version'
