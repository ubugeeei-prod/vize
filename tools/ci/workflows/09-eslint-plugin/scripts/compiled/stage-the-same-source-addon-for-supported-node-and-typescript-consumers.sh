#!/bin/sh
# Generated from '../stage-the-same-source-addon-for-supported-node-and-typescript-consumers.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../stage-the-same-source-addon-for-supported-node-and-typescript-consumers.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../stage-the-same-source-addon-for-supported-node-and-typescript-consumers.ush' --checked-in -o 'stage-the-same-source-addon-for-supported-node-and-typescript-consumers.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'stage-the-same-source-addon-for-supported-node-and-typescript-consumers.ush'
__ush_origin='../stage-the-same-source-addon-for-supported-node-and-typescript-consumers.ush'

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
  -h|--help) printf '%s' 'stage-the-same-source-addon-for-supported-node-and-typescript-consumers.ush

Usage:
  stage-the-same-source-addon-for-supported-node-and-typescript-consumers.ush [--help]
  stage-the-same-source-addon-for-supported-node-and-typescript-consumers.ush [--man [ITEM]]
  stage-the-same-source-addon-for-supported-node-and-typescript-consumers.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
stage-the-same-source-addon-for-supported-node-and-typescript-consumers.ush

SYNOPSIS
stage-the-same-source-addon-for-supported-node-and-typescript-consumers.ush [--help]
stage-the-same-source-addon-for-supported-node-and-typescript-consumers.ush [--man [ITEM]]
stage-the-same-source-addon-for-supported-node-and-typescript-consumers.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_std_path_join_parts() {
  __ush_left=$1
  __ush_right=$2
  if [ -z "$__ush_left" ]; then
    printf '%s' "$__ush_right"
    return 0
  fi
  if [ -z "$__ush_right" ]; then
    printf '%s' "$__ush_left"
    return 0
  fi
  while [ "$__ush_left" != "/" ] && [ "${__ush_left%/}" != "$__ush_left" ]; do
    __ush_left=${__ush_left%/}
  done
  while [ "${__ush_right#/}" != "$__ush_right" ]; do
    __ush_right=${__ush_right#/}
  done
  case "$__ush_left" in
    /) printf '/%s' "$__ush_right" ;;
    *) printf '%s/%s' "$__ush_left" "$__ush_right" ;;
  esac
}

__ush_std_path_resolve() {
  __ush_value=$1
  case "$__ush_value" in
    __ush_pathref__:cwd:*)
      __ush_std_path_join_parts "$(pwd)" "${__ush_value#__ush_pathref__:cwd:}"
      ;;
    __ush_pathref__:source:*)
      __ush_base=${__ush_source_dir:-}
      if [ -z "$__ush_base" ]; then
        __ush_base=$(pwd)
      fi
      __ush_std_path_join_parts "$__ush_base" "${__ush_value#__ush_pathref__:source:}"
      ;;
    *) printf '%s' "$__ush_value" ;;
  esac
}

ush_fn_std__path__from_source() {
  # The source's directory is fixed for the whole run, so a source-relative path
  # is resolved at once: the value is a real path wherever it is passed.
  case "$1" in
    ''|/*) printf '%s' "$1" ;;
    *) __ush_std_path_resolve "__ush_pathref__:source:$1" ;;
  esac
}

__ush_at 5 'node (from_source "stage-the-same-source-addon-for-supported-node-and-typescript-consumers.ts")'
__ush_value_0="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__from_source 'stage-the-same-source-addon-for-supported-node-and-typescript-consumers.ts')" || exit "$?"
command node "${__ush_value_0}"
