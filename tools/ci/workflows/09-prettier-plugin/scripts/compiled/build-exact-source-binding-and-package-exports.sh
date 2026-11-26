#!/bin/sh
# Generated from '../build-exact-source-binding-and-package-exports.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../build-exact-source-binding-and-package-exports.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../build-exact-source-binding-and-package-exports.ush' --checked-in -o 'build-exact-source-binding-and-package-exports.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'build-exact-source-binding-and-package-exports.ush'
__ush_origin='../build-exact-source-binding-and-package-exports.ush'

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
  -h|--help) printf '%s' 'build-exact-source-binding-and-package-exports.ush

Usage:
  build-exact-source-binding-and-package-exports.ush [--help]
  build-exact-source-binding-and-package-exports.ush [--man [ITEM]]
  build-exact-source-binding-and-package-exports.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
build-exact-source-binding-and-package-exports.ush

SYNOPSIS
build-exact-source-binding-and-package-exports.ush [--help]
build-exact-source-binding-and-package-exports.ush [--man [ITEM]]
build-exact-source-binding-and-package-exports.ush [--complete [PREFIX]]
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

ush_fn_std__path__resolve() {
  __ush_std_path_resolve "$1"
}

__ush_std_fs_path() {
  __ush_path=$(ush_fn_std__path__resolve "$1" && printf .) || return "$?"
  __ush_path=${__ush_path%.}
  [ -n "$__ush_path" ] || { printf '%s\n' 'std::fs: path must not be empty' >&2; return 1; }
  case $__ush_path in .|..|/*|./*|../*) ;; *) __ush_path=./$__ush_path ;; esac
}

ush_fn_std__process__chdir() {
  __ush_std_fs_path "$1" || return "$?"
  CDPATH='' cd "$__ush_path" || return "$?"
}

__ush_at 5 'let source_sha = $env::SOURCE_SHA'
__ush_env_0=${SOURCE_SHA-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable SOURCE_SHA is not set or is empty'; exit 1; fi
__ush_binding_5_source_sha="${__ush_env_0}"
# shellcheck disable=SC2034
source_sha="${__ush_binding_5_source_sha}"
__ush_at 6 'let checkout = git rev-parse HEAD'
__ush_value_1="$(__ush_return_path='' __ush_capture_return='1' command git 'rev-parse' 'HEAD')" || exit "$?"
__ush_binding_6_checkout="${__ush_value_1}"
# shellcheck disable=SC2034
checkout="${__ush_binding_6_checkout}"
__ush_at 7 'if checkout != source_sha {'
if {
[ "$(if [ "${__ush_binding_6_checkout}" != "${__ush_binding_5_source_sha}" ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 8 'fail 1 $"The checkout is at $checkout, not at SOURCE_SHA $source_sha"'
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'The checkout is at '"${__ush_binding_6_checkout}"', not at SOURCE_SHA '"${__ush_binding_5_source_sha}"
  exit "$__ush_status"
fi
__ush_at 10 'let node_version = node (from_source "build-exact-source-binding-and-package-exports.cjs")'
__ush_value_2="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__from_source 'build-exact-source-binding-and-package-exports.cjs')" || exit "$?"
__ush_value_3="$(__ush_return_path='' __ush_capture_return='1' command node "${__ush_value_2}")" || exit "$?"
__ush_binding_10_node_version="${__ush_value_3}"
# shellcheck disable=SC2034
node_version="${__ush_binding_10_node_version}"
__ush_at 11 'if node_version != "24.14.0" {'
if {
[ "$(if [ "${__ush_binding_10_node_version}" != '24.14.0' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 12 'fail 1 $"The binding is built on Node 24.14.0, not on $node_version"'
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'The binding is built on Node 24.14.0, not on '"${__ush_binding_10_node_version}"
  exit "$__ush_status"
fi
__ush_at 14 'vp run --workspace-root build:native:test'
command vp 'run' '--workspace-root' 'build:native:test'
__ush_at 15 'cd "npm/formatter/prettier" {'
__ush_cwd_4=$PWD
ush_fn_std__process__chdir 'npm/formatter/prettier' || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
  __ush_at 16 'vp check src test vite.config.ts'
  command vp 'check' 'src' 'test' 'vite.config.ts'
  __ush_at 17 'vp pack'
  command vp 'pack'
cd "$__ush_cwd_4" || { __ush_status=$?; __ush_report "$__ush_status" 'cannot return to the directory the cd block started in'; exit "$__ush_status"; }
