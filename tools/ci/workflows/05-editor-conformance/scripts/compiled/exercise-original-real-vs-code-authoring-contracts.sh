#!/bin/sh
# Generated from '../exercise-original-real-vs-code-authoring-contracts.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../exercise-original-real-vs-code-authoring-contracts.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../exercise-original-real-vs-code-authoring-contracts.ush' --checked-in -o 'exercise-original-real-vs-code-authoring-contracts.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'exercise-original-real-vs-code-authoring-contracts.ush'
__ush_origin='../exercise-original-real-vs-code-authoring-contracts.ush'

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
  -h|--help) printf '%s' 'exercise-original-real-vs-code-authoring-contracts.ush

Usage:
  exercise-original-real-vs-code-authoring-contracts.ush [--help]
  exercise-original-real-vs-code-authoring-contracts.ush [--man [ITEM]]
  exercise-original-real-vs-code-authoring-contracts.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
exercise-original-real-vs-code-authoring-contracts.ush

SYNOPSIS
exercise-original-real-vs-code-authoring-contracts.ush [--help]
exercise-original-real-vs-code-authoring-contracts.ush [--man [ITEM]]
exercise-original-real-vs-code-authoring-contracts.ush [--complete [PREFIX]]
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

__ush_std_fs_path() {
  __ush_path=$(ush_fn_std__path__resolve "$1" && printf .) || return "$?"
  __ush_path=${__ush_path%.}
  [ -n "$__ush_path" ] || { printf '%s\n' 'std::fs: path must not be empty' >&2; return 1; }
  case $__ush_path in .|..|/*|./*|../*) ;; *) __ush_path=./$__ush_path ;; esac
}

__ush_at 6 'let evidence = $"$env::RUNNER_TEMP/rich-host-evidence"'
__ush_env_0=${RUNNER_TEMP-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_6_evidence="${__ush_env_0}"'/rich-host-evidence'
# shellcheck disable=SC2034
evidence="${__ush_binding_6_evidence}"
__ush_at 8 'mkdir_p $evidence'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__mkdir_p "${__ush_binding_6_evidence}" )
__ush_at 9 'cp $"$env::RUNNER_TEMP/vize-server/vize-producer.txt" $"$evidence/"'
__ush_env_1=${RUNNER_TEMP-}
if [ -z "${__ush_env_1}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
command cp "${__ush_env_1}"'/vize-server/vize-producer.txt' "${__ush_binding_6_evidence}"'/'
__ush_at 10 'sha256sum $env::VIZE_SERVER_PATH | to_file $"$evidence/consumed-server.sha256"'
__ush_env_2=${VIZE_SERVER_PATH-}
if [ -z "${__ush_env_2}" ]; then __ush_report 1 'environment variable VIZE_SERVER_PATH is not set or is empty'; exit 1; fi
__ush_std_fs_path "${__ush_binding_6_evidence}"'/consumed-server.sha256' || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
__ush_path_3=$__ush_path
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (command sha256sum "${__ush_env_2}") 7>&- >"$__ush_path_3"; printf ' 1:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: sha256sum $env::VIZE_SERVER_PATH | to_file $"$evidence/co...' || exit "$?"
__ush_at 11 'git rev-parse HEAD | to_file $"$evidence/harness-sha.txt"'
__ush_std_fs_path "${__ush_binding_6_evidence}"'/harness-sha.txt' || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
__ush_path_4=$__ush_path
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (command git 'rev-parse' 'HEAD') 7>&- >"$__ush_path_4"; printf ' 1:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: git rev-parse HEAD | to_file $"$evidence/harness-sha.txt"' || exit "$?"
__ush_at 12 'xvfb_run -a vp run --workspace-root test:vscode-extension:host-real'
command xvfb-run '-a' 'vp' 'run' '--workspace-root' 'test:vscode-extension:host-real'
