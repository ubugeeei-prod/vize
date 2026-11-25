#!/bin/sh
# Generated from '../build-docs.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../build-docs.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../build-docs.ush' --checked-in -o 'build-docs.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'build-docs.ush'
__ush_origin='../build-docs.ush'

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
  -h|--help) printf '%s' 'build-docs.ush

Usage:
  build-docs.ush [--help]
  build-docs.ush [--man [ITEM]]
  build-docs.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
build-docs.ush

SYNOPSIS
build-docs.ush [--help]
build-docs.ush [--man [ITEM]]
build-docs.ush [--complete [PREFIX]]
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

ush_fn_std__path__resolve() {
  __ush_std_path_resolve "$1"
}

__ush_std_fs_path() {
  __ush_path=$(ush_fn_std__path__resolve "$1" && printf .) || return "$?"
  __ush_path=${__ush_path%.}
  [ -n "$__ush_path" ] || { printf '%s\n' 'std::fs: path must not be empty' >&2; return 1; }
  case $__ush_path in .|..|/*|./*|../*) ;; *) __ush_path=./$__ush_path ;; esac
}

ush_fn_std__fs__size() {
  __ush_std_fs_path "$1" || return "$?"
  __ush_size=$(wc -c < "$__ush_path") || return "$?"
  __ush_size=${__ush_size#"${__ush_size%%[![:space:]]*}"}
  printf '%s' "$__ush_size"
}

__ush_at 5 'let previews = "docs-render-evidence/component-previews"'
__ush_binding_5_previews='docs-render-evidence/component-previews'
# shellcheck disable=SC2034
previews="${__ush_binding_5_previews}"
__ush_at 7 'vp run --no-cache --filter ./docs build'
command vp 'run' '--no-cache' '--filter' './docs' 'build'
__ush_at 8 'for evidence in ["browser-evidence.json", "unavailable-storage-evidence.json"] {'
__ush_match_0__len='2'
# shellcheck disable=SC2034
__ush_match_0__0='browser-evidence.json'
# shellcheck disable=SC2034
__ush_match_0__1='unavailable-storage-evidence.json'
__ush_for_list_index_1=0
while [ "${__ush_for_list_index_1}" -lt "${__ush_match_0__len}" ]; do
  __ush_for_value_2=''
  eval "__ush_for_value_2=\"\${__ush_match_0__${__ush_for_list_index_1}}\""
  evidence="${__ush_for_value_2}"
  __ush_for_list_index_1=$((__ush_for_list_index_1 + 1))
  __ush_at 9 'if (size $"$previews/$evidence") == 0 {'
  if {
  __ush_value_3="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__fs__size "${__ush_binding_5_previews}"'/'"${evidence}")" || exit "$?"
  [ "$(if [ "${__ush_value_3}" -eq 0 ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
  }; then
    __ush_at 10 'fail 1 $"The docs build recorded an empty $evidence"'
    __ush_status=1
    if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
    __ush_report "$__ush_status" 'The docs build recorded an empty '"${evidence}"
    exit "$__ush_status"
  fi
done
