#!/bin/sh
# Generated from '../check-authored-positions-and-existing-rspack-behavior.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../check-authored-positions-and-existing-rspack-behavior.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../check-authored-positions-and-existing-rspack-behavior.ush' --checked-in -o 'check-authored-positions-and-existing-rspack-behavior.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'check-authored-positions-and-existing-rspack-behavior.ush'
__ush_origin='../check-authored-positions-and-existing-rspack-behavior.ush'

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
  -h|--help) printf '%s' 'check-authored-positions-and-existing-rspack-behavior.ush

Usage:
  check-authored-positions-and-existing-rspack-behavior.ush [--help]
  check-authored-positions-and-existing-rspack-behavior.ush [--man [ITEM]]
  check-authored-positions-and-existing-rspack-behavior.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
check-authored-positions-and-existing-rspack-behavior.ush

SYNOPSIS
check-authored-positions-and-existing-rspack-behavior.ush [--help]
check-authored-positions-and-existing-rspack-behavior.ush [--man [ITEM]]
check-authored-positions-and-existing-rspack-behavior.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

# Report the leftmost failing pipeline stage. $1 holds one ` INDEX:STATUS`
# per stage in completion order; each further argument is the diagnostic
# for that stage. Every stage holds its end of the pipe until it has
# reported, so a stage whose reader reported first was cut off by a
# reader that stopped early, as `head` does. Its status is SIGPIPE (141),
# or a write error where the caller ignores SIGPIPE; either way the
# reader's own status decides. An empty diagnostic, as in a condition or
# `status_of`, sets the status without reporting it.
__ush_pipe_check() {
  __ush_pipe_statuses=" $1 "
  shift
  __ush_pipe_index=0
  for __ush_pipe_message do
    __ush_pipe_index=$((__ush_pipe_index + 1))
    case $__ush_pipe_statuses in
      *" $__ush_pipe_index:0 "*) continue ;;
      *" $__ush_pipe_index:"*) ;;
      *)
        [ -z "$__ush_pipe_message" ] || __ush_report 1 "$__ush_pipe_message (no exit status)"
        return 1 ;;
    esac
    __ush_pipe_code=${__ush_pipe_statuses#*" $__ush_pipe_index:"}
    __ush_pipe_code=${__ush_pipe_code%% *}
    __ush_pipe_earlier=${__ush_pipe_statuses%%" $__ush_pipe_index:"*}
    case $__ush_pipe_earlier in
      *" $((__ush_pipe_index + 1)):"*) continue ;;
    esac
    [ -z "$__ush_pipe_message" ] || __ush_report "$__ush_pipe_code" "$__ush_pipe_message"
    return "$__ush_pipe_code"
  done
}

ush_fn_std__fs__mkdir_p() {
  __ush_path=$(ush_fn_std__path__resolve "$1")
  mkdir -p "$__ush_path"
}

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

__ush_at 5 'mkdir_p $env::VIZE_TURBOPACK_ARTIFACTS'
__ush_env_0=${VIZE_TURBOPACK_ARTIFACTS-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable VIZE_TURBOPACK_ARTIFACTS is not set or is empty'; exit 1; fi
( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__mkdir_p "${__ush_env_0}" )
__ush_at 6 'node --test \'
command node '--test' 'tests/tooling/turbopack-workflow.test.ts' 'npm/builder/turbopack/test/consumer-packaging.test.mjs'
__ush_at 10 'node --test \'
__ush_env_1=${VIZE_TURBOPACK_ARTIFACTS-}
if [ -z "${__ush_env_1}" ]; then __ush_report 1 'environment variable VIZE_TURBOPACK_ARTIFACTS is not set or is empty'; exit 1; fi
__ush_value_2="${__ush_env_1}"
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (command node '--test' 'npm/builder/turbopack/src/*.test.ts' 'npm/builder/shared/*.test.ts' 'npm/builder/rspack/src/shared/source-map.test.ts' 'npm/builder/rspack/src/shared/output.test.ts' 'npm/builder/rspack/src/shared/hotReload.test.ts') 7>&-; printf ' 1:%s' "$?" >&7; } |
  { (command tee "${__ush_value_2}"'/source-unit.tap') 7>&-; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: node --test npm/builder/turbopack/src/*.test.ts npm/build...' 'pipeline stage 2 failed: tee $env::VIZE_TURBOPACK_ARTIFACTS/source-unit.tap' || exit "$?"
