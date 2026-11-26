#!/bin/sh
# Generated from '../exercise-both-packaged-neovim-headless-suites.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../exercise-both-packaged-neovim-headless-suites.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../exercise-both-packaged-neovim-headless-suites.ush' --checked-in -o 'exercise-both-packaged-neovim-headless-suites.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'exercise-both-packaged-neovim-headless-suites.ush'
__ush_origin='../exercise-both-packaged-neovim-headless-suites.ush'

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
  -h|--help) printf '%s' 'exercise-both-packaged-neovim-headless-suites.ush

Usage:
  exercise-both-packaged-neovim-headless-suites.ush [--help]
  exercise-both-packaged-neovim-headless-suites.ush [--man [ITEM]]
  exercise-both-packaged-neovim-headless-suites.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
exercise-both-packaged-neovim-headless-suites.ush

SYNOPSIS
exercise-both-packaged-neovim-headless-suites.ush [--help]
exercise-both-packaged-neovim-headless-suites.ush [--man [ITEM]]
exercise-both-packaged-neovim-headless-suites.ush [--complete [PREFIX]]
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

__ush_std_path_dirname_ref() {
  __ush_value=$1
  case "$__ush_value" in
    __ush_pathref__:cwd:*)
      printf '__ush_pathref__:cwd:%s' "$(dirname "${__ush_value#__ush_pathref__:cwd:}")"
      ;;
    __ush_pathref__:source:*)
      printf '__ush_pathref__:source:%s' "$(dirname "${__ush_value#__ush_pathref__:source:}")"
      ;;
    *) dirname "$__ush_value" ;;
  esac
}

ush_fn_std__path__dirname() {
  __ush_std_path_dirname_ref "$1"
}

__ush_at 5 'let pinned = $env::VIZE_TEST_NVIM_PATH'
__ush_env_0=${VIZE_TEST_NVIM_PATH-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable VIZE_TEST_NVIM_PATH is not set or is empty'; exit 1; fi
__ush_binding_5_pinned="${__ush_env_0}"
# shellcheck disable=SC2034
pinned="${__ush_binding_5_pinned}"
__ush_at 7 'export PATH = $"${dirname $pinned}:$env::PATH"'
__ush_value_1="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__dirname "${__ush_binding_5_pinned}")" || exit "$?"
__ush_env_2=${PATH-}
if [ -z "${__ush_env_2}" ]; then __ush_report 1 'environment variable PATH is not set or is empty'; exit 1; fi
export PATH="${__ush_value_1}"':'"${__ush_env_2}"
__ush_at 8 'if (command -v nvim) != $pinned {'
if {
__ush_value_3="$(__ush_return_path='' __ush_capture_return='1' command command '-v' 'nvim')" || exit "$?"
[ "$(if [ "${__ush_value_3}" != "${__ush_binding_5_pinned}" ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 9 'fail 1 $"The nvim on PATH is not the pinned $pinned"'
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'The nvim on PATH is not the pinned '"${__ush_binding_5_pinned}"
  exit "$__ush_status"
fi
__ush_at 11 'command -v nvim'
command command '-v' 'nvim'
__ush_at 12 'nvim --version | head -1'
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (command nvim '--version') 7>&-; printf ' 1:%s' "$?" >&7; } |
  { (command head -1) 7>&-; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: nvim --version' 'pipeline stage 2 failed: head -1' || exit "$?"
__ush_at 13 'vp run --workspace-root test:nvim-extension:headless'
command vp 'run' '--workspace-root' 'test:nvim-extension:headless'
