#!/bin/sh
# Generated from '../check-the-prefix-theorem-with-lean.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../check-the-prefix-theorem-with-lean.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../check-the-prefix-theorem-with-lean.ush' --checked-in -o 'check-the-prefix-theorem-with-lean.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'check-the-prefix-theorem-with-lean.ush'
__ush_origin='../check-the-prefix-theorem-with-lean.ush'

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
  -h|--help) printf '%s' 'check-the-prefix-theorem-with-lean.ush

Usage:
  check-the-prefix-theorem-with-lean.ush [--help]
  check-the-prefix-theorem-with-lean.ush [--man [ITEM]]
  check-the-prefix-theorem-with-lean.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
check-the-prefix-theorem-with-lean.ush

SYNOPSIS
check-the-prefix-theorem-with-lean.ush [--help]
check-the-prefix-theorem-with-lean.ush [--man [ITEM]]
check-the-prefix-theorem-with-lean.ush [--complete [PREFIX]]
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

ush_fn_std__pack() {
  __ush_count=0
  eval "__ush_count=\${${1}__len}"
  __ush_i=0
  while [ "$__ush_i" -lt "$__ush_count" ]; do
    eval "__ush_element=\${${1}__${__ush_i}}"
    printf x
    printf '%s' "$__ush_element" | od -A n -v -t x1 | tr -d ' \n'
    printf ' '
    __ush_i=$((__ush_i + 1))
  done
}

ush_fn_std__unpack() {
  __ush_i=0
  for __ush_hex in $2; do
    __ush_element=$(LC_ALL=C awk -v s="${__ush_hex#x}" 'BEGIN {
      digits = "0123456789abcdef"
      for (i = 1; i <= length(s); i += 2)
        printf "%c", (index(digits, substr(s,i,1))-1)*16 + index(digits,substr(s,i+1,1))-1
      printf "."
    }')
    __ush_element=${__ush_element%.}
    eval "${1}__${__ush_i}=\$__ush_element"
    __ush_i=$((__ush_i + 1))
  done
  eval "${1}__len=\$__ush_i"
}

ush_fn_std__process__exec() {
  __ush_program=$1
  __ush_args__len=0
  __ush_arg=''
  ush_fn_std__unpack __ush_args "$2" || return "$?"
  set --
  __ush_index=0
  while [ "$__ush_index" -lt "$__ush_args__len" ]; do
    eval "__ush_arg=\${__ush_args__${__ush_index}}"
    set -- "$@" "$__ush_arg"
    __ush_index=$((__ush_index + 1))
  done
  "$__ush_program" "$@"
}

__ush_at 5 'let theorem = $"$env::GITHUB_WORKSPACE/tests/formal/template-lint/Selection.lean"'
__ush_env_0=${GITHUB_WORKSPACE-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable GITHUB_WORKSPACE is not set or is empty'; exit 1; fi
__ush_binding_5_theorem="${__ush_env_0}"'/tests/formal/template-lint/Selection.lean'
# shellcheck disable=SC2034
theorem="${__ush_binding_5_theorem}"
__ush_at 7 'curl -fsSL --retry 3 -o lean.tar.zst https://github.com/leanprover/lean4/releases/download/v4.33.1/lean-4.33.1-linux.tar.zst'
command curl '-fsSL' '--retry' 3 '-o' 'lean.tar.zst' 'https://github.com/leanprover/lean4/releases/download/v4.33.1/lean-4.33.1-linux.tar.zst'
__ush_at 8 '"890afd185370f85666025b883914ab4f4b339136f8c96167b69cfb62aecaf235  lean.tar.zst\n" | sha256sum --check'
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (printf '%s' '890afd185370f85666025b883914ab4f4b339136f8c96167b69cfb62aecaf235  lean.tar.zst'"${__ush_lf}"'') 7>&-; printf ' 1:%s' "$?" >&7; } |
  { (command sha256sum '--check') 7>&-; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: "890afd185370f85666025b883914ab4f4b339136f8c96167b69cfb62...' 'pipeline stage 2 failed: sha256sum --check' || exit "$?"
__ush_at 9 'tar --zstd -xf lean.tar.zst'
command tar '--zstd' '-xf' 'lean.tar.zst'
__ush_at 10 'exec lean-4.33.1-linux/bin/lean ["-DwarningAsError=true", theorem] | tee theorem.log'
__ush_match_1__len='2'
# shellcheck disable=SC2034
__ush_match_1__0='-DwarningAsError=true'
# shellcheck disable=SC2034
__ush_match_1__1="${__ush_binding_5_theorem}"
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (ush_fn_std__process__exec 'lean-4.33.1-linux/bin/lean' "$(ush_fn_std__pack __ush_match_1)") 7>&-; printf ' 1:%s' "$?" >&7; } |
  { (command tee 'theorem.log') 7>&-; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: exec lean-4.33.1-linux/bin/lean ["-DwarningAsError=true",...' 'pipeline stage 2 failed: tee theorem.log' || exit "$?"
__ush_at 11 'grep -Fx "'"'"'Davinci.TemplateLint.fold_prefix'"'"' depends on axioms: [propext]" theorem.log'
command grep '-Fx' ''"'"'Davinci.TemplateLint.fold_prefix'"'"' depends on axioms: [propext]' 'theorem.log'
__ush_at 12 'grep -Fx "'"'"'Davinci.TemplateLint.labels_exact'"'"' depends on axioms: [propext, Classical.choice, Quot.sound]" theorem.log'
command grep '-Fx' ''"'"'Davinci.TemplateLint.labels_exact'"'"' depends on axioms: [propext, Classical.choice, Quot.sound]' 'theorem.log'
