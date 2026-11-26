#!/bin/sh
# Generated from '../install-the-frozen-nuxt-5-production-reproduction.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../install-the-frozen-nuxt-5-production-reproduction.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../install-the-frozen-nuxt-5-production-reproduction.ush' --checked-in -o 'install-the-frozen-nuxt-5-production-reproduction.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'install-the-frozen-nuxt-5-production-reproduction.ush'
__ush_origin='../install-the-frozen-nuxt-5-production-reproduction.ush'

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
  -h|--help) printf '%s' 'install-the-frozen-nuxt-5-production-reproduction.ush

Usage:
  install-the-frozen-nuxt-5-production-reproduction.ush [--help]
  install-the-frozen-nuxt-5-production-reproduction.ush [--man [ITEM]]
  install-the-frozen-nuxt-5-production-reproduction.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
install-the-frozen-nuxt-5-production-reproduction.ush

SYNOPSIS
install-the-frozen-nuxt-5-production-reproduction.ush [--help]
install-the-frozen-nuxt-5-production-reproduction.ush [--man [ITEM]]
install-the-frozen-nuxt-5-production-reproduction.ush [--complete [PREFIX]]
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

__ush_at 7 'let evidence = $"$env::RUNNER_TEMP/nuxt5-runtime-proof"'
__ush_env_0=${RUNNER_TEMP-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_7_evidence="${__ush_env_0}"'/nuxt5-runtime-proof'
# shellcheck disable=SC2034
evidence="${__ush_binding_7_evidence}"
__ush_at 8 'let describe_runtime = from_source "install-the-frozen-nuxt-5-production-reproduction.cjs"'
__ush_value_1="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__from_source 'install-the-frozen-nuxt-5-production-reproduction.cjs')" || exit "$?"
__ush_binding_8_describe_runtime="${__ush_value_1}"
# shellcheck disable=SC2034
describe_runtime="${__ush_binding_8_describe_runtime}"
__ush_at 10 'mkdir_p $evidence'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__mkdir_p "${__ush_binding_7_evidence}" )
__ush_at 11 'node $describe_runtime | to_file $evidence/runtime.json'
__ush_value_2="${__ush_binding_7_evidence}"
__ush_std_fs_path "${__ush_value_2}"'/runtime.json' || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
__ush_path_3=$__ush_path
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (command node "${__ush_binding_8_describe_runtime}") 7>&- >"$__ush_path_3"; printf ' 1:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: node $describe_runtime | to_file $evidence/runtime.json' || exit "$?"
__ush_at 12 'npm ci --legacy-peer-deps --no-audit --no-fund |& tee $evidence/install.log'
__ush_value_4="${__ush_binding_7_evidence}"
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (command npm 'ci' '--legacy-peer-deps' '--no-audit' '--no-fund') 7>&- 2>&1; printf ' 1:%s' "$?" >&7; } |
  { (command tee "${__ush_value_4}"'/install.log') 7>&-; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: npm ci --legacy-peer-deps --no-audit --no-fund' 'pipeline stage 2 failed: tee $evidence/install.log' || exit "$?"
