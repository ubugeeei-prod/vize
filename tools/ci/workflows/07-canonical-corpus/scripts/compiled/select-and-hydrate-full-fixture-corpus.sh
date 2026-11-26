#!/bin/sh
# Generated from '../select-and-hydrate-full-fixture-corpus.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../select-and-hydrate-full-fixture-corpus.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../select-and-hydrate-full-fixture-corpus.ush' --checked-in -o 'select-and-hydrate-full-fixture-corpus.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'select-and-hydrate-full-fixture-corpus.ush'
__ush_origin='../select-and-hydrate-full-fixture-corpus.ush'

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
  -h|--help) printf '%s' 'select-and-hydrate-full-fixture-corpus.ush

Usage:
  select-and-hydrate-full-fixture-corpus.ush [--help]
  select-and-hydrate-full-fixture-corpus.ush [--man [ITEM]]
  select-and-hydrate-full-fixture-corpus.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
select-and-hydrate-full-fixture-corpus.ush

SYNOPSIS
select-and-hydrate-full-fixture-corpus.ush [--help]
select-and-hydrate-full-fixture-corpus.ush [--man [ITEM]]
select-and-hydrate-full-fixture-corpus.ush [--complete [PREFIX]]
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

__ush_at 4 'node tools/support/compat/fixtures/davinci-dom-corpus-workflow.ts hydrate |& tee real-project-davinci-dom-corpus/initial-hydration.log'
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (command node 'tools/support/compat/fixtures/davinci-dom-corpus-workflow.ts' 'hydrate') 7>&- 2>&1; printf ' 1:%s' "$?" >&7; } |
  { (command tee 'real-project-davinci-dom-corpus/initial-hydration.log') 7>&-; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: node tools/support/compat/fixtures/davinci-dom-corpus-wor...' 'pipeline stage 2 failed: tee real-project-davinci-dom-corpus/initial-hydration.log' || exit "$?"
__ush_at 5 'node tools/support/compat/github/canonical-corpus-observer.mjs rehydrate'
command node 'tools/support/compat/github/canonical-corpus-observer.mjs' 'rehydrate'
