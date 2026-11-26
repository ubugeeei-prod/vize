#!/bin/sh
# Generated from '../record-the-real-server-source-and-executable-identity.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../record-the-real-server-source-and-executable-identity.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../record-the-real-server-source-and-executable-identity.ush' --checked-in -o 'record-the-real-server-source-and-executable-identity.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'record-the-real-server-source-and-executable-identity.ush'
__ush_origin='../record-the-real-server-source-and-executable-identity.ush'

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
  -h|--help) printf '%s' 'record-the-real-server-source-and-executable-identity.ush

Usage:
  record-the-real-server-source-and-executable-identity.ush [--help]
  record-the-real-server-source-and-executable-identity.ush [--man [ITEM]]
  record-the-real-server-source-and-executable-identity.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
record-the-real-server-source-and-executable-identity.ush

SYNOPSIS
record-the-real-server-source-and-executable-identity.ush [--help]
record-the-real-server-source-and-executable-identity.ush [--man [ITEM]]
record-the-real-server-source-and-executable-identity.ush [--complete [PREFIX]]
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

ush_fn_std__process__chdir() {
  __ush_std_fs_path "$1" || return "$?"
  CDPATH='' cd "$__ush_path" || return "$?"
}

# raises: unknown
ush_fn_producer_record() {
  __ush_fn_producer_record_arg_0="${1}"
  # shellcheck disable=SC2034
  source="${__ush_fn_producer_record_arg_0}"
  __ush_at 6 'echo $"producer=$source"'
  printf '%s\n' 'producer='"${__ush_fn_producer_record_arg_0}" || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
  __ush_at 7 'echo "recipe=cargo build --profile ci -p vize"'
  printf '%s\n' 'recipe=cargo build --profile ci -p vize' || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
  __ush_at 8 'rustc --version --verbose'
  command rustc '--version' '--verbose' || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
  __ush_at 9 'cargo --version'
  command cargo '--version' || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
  __ush_at 10 'sha256sum rust-toolchain.toml Cargo.lock target/ci/vize'
  command sha256sum 'rust-toolchain.toml' 'Cargo.lock' 'target/ci/vize' || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
}

__ush_at 13 'let source = git rev-parse HEAD'
__ush_value_0="$(__ush_return_path='' __ush_capture_return='1' command git 'rev-parse' 'HEAD')" || exit "$?"
__ush_binding_13_source="${__ush_value_0}"
# shellcheck disable=SC2034
source="${__ush_binding_13_source}"
__ush_at 14 'if $source != $env::GITHUB_SHA {'
if {
__ush_env_1=${GITHUB_SHA-}
if [ -z "${__ush_env_1}" ]; then __ush_report 1 'environment variable GITHUB_SHA is not set or is empty'; exit 1; fi
[ "$(if [ "${__ush_binding_13_source}" != "${__ush_env_1}" ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 15 'fail 1 $"The checkout is $source, not the commit under test $env::GITHUB_SHA"'
  __ush_env_2=${GITHUB_SHA-}
  if [ -z "${__ush_env_2}" ]; then __ush_report 1 'environment variable GITHUB_SHA is not set or is empty'; exit 1; fi
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'The checkout is '"${__ush_binding_13_source}"', not the commit under test '"${__ush_env_2}"
  exit "$__ush_status"
fi
__ush_at 17 'producer_record $source | to_file target/ci/vize-producer.txt'
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { ( __ush_capture_return='0' __ush_return_path=''; ush_fn_producer_record "${__ush_binding_13_source}" ) 7>&- >'target/ci/vize-producer.txt'; printf ' 1:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: producer_record $source | to_file target/ci/vize-producer...' || exit "$?"
__ush_at 18 'cd "target/ci" {'
__ush_cwd_3=$PWD
ush_fn_std__process__chdir 'target/ci' || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
  __ush_at 19 'sha256sum vize | to_file vize.sha256'
  { __ush_pipe=$(
    exec 7>&1 >&8 8>&-
    set +e
    { (command sha256sum 'vize') 7>&- >'vize.sha256'; printf ' 1:%s' "$?" >&7; }
  ); } 8>&1
  __ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: sha256sum vize | to_file vize.sha256' || exit "$?"
cd "$__ush_cwd_3" || { __ush_status=$?; __ush_report "$__ush_status" 'cannot return to the directory the cd block started in'; exit "$__ush_status"; }
