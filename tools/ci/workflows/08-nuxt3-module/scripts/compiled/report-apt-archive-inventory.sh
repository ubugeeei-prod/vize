#!/bin/sh
# Generated from '../report-apt-archive-inventory.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../report-apt-archive-inventory.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../report-apt-archive-inventory.ush' --checked-in -o 'report-apt-archive-inventory.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'report-apt-archive-inventory.ush'
__ush_origin='../report-apt-archive-inventory.ush'

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

ush_fn_std__path__script() {
  # The running script file itself, wherever it was copied: `from_source`
  # names the ush source instead.
  __ush_script=${__ush_entrypoint:-$0}
  case $__ush_script in
    */*) ;;
    *) [ -f "$__ush_script" ] || __ush_script=$(command -v -- "$__ush_script") || return 1 ;;
  esac
  __ush_script_dir=$(CDPATH='' cd -P "$(dirname "$__ush_script")" && pwd -P) || return 1
  printf '%s/%s' "$__ush_script_dir" "$(basename "$__ush_script")"
}

ush_fn_std__path__dirname() {
  __ush_std_path_dirname_ref "$1"
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

__ush_std_encode_args() {
  for __ush_arg do
    printf x
    printf '%s' "$__ush_arg" | od -A n -v -t x1 | tr -d ' \n'
    printf ' '
  done
}

ush_fn_std__process__args() {
  # Keep the encoder linked even when args is the only process helper used.
  if [ "${__ush_argv+x}" != x ]; then __ush_std_encode_args; else printf '%s' "$__ush_argv"; fi
}

ush_fn_std__process__arg() {
  __ush_requested=$1
  __ush_args__len=0
  ush_fn_std__unpack __ush_args "$(ush_fn_std__process__args)"
  case $__ush_requested in ''|*[!0-9]*) return 1 ;; esac
  if [ "$__ush_requested" -lt "$__ush_args__len" ]; then
    eval "printf '%s' \"\${__ush_args__${__ush_requested}}\""
  fi
}

__ush_argv="$(__ush_std_encode_args "$@")"
__ush_at 7 'let moment = match (arg 0) {'
__ush_value_1="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__process__arg 0)" || exit "$?"
__ush_match_0="${__ush_value_1}"
case "${__ush_match_0}" in
  ''|'after-update')
    __ush_at 8 '"" | "after-update" => "after package-index update"'
    __ush_binding_7_moment='after package-index update'
    ;;
  'before-install')
    __ush_at 9 '"before-install" => "before the original Chromium installation"'
    __ush_binding_7_moment='before the original Chromium installation'
    ;;
  *)
    other="${__ush_match_0-}"
    __ush_at 10 'other => fail 2 $"Unknown inventory moment: $other"'
    __ush_status=2
    if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
    __ush_report "$__ush_status" 'Unknown inventory moment: '"${other}"
    exit "$__ush_status"
    ;;
esac
# shellcheck disable=SC2034
moment="${__ush_binding_7_moment}"
__ush_at 13 'echo $"Nuxt3 archive inventory $moment"'
printf '%s\n' 'Nuxt3 archive inventory '"${__ush_binding_7_moment}"
__ush_at 15 'find (dirname (script)) -maxdepth 1 -type f -name *.deb -print0 |'
__ush_value_2="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__script)" || exit "$?"
__ush_value_3="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__dirname "${__ush_value_2}")" || exit "$?"
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (command find "${__ush_value_3}" '-maxdepth' 1 '-type' 'f' '-name' '*.deb' '-print0') 7>&-; printf ' 1:%s' "$?" >&7; } |
  { (command sort '-z') 7>&-; printf ' 2:%s' "$?" >&7; } |
  { (command xargs '-0' '-r' 'sha256sum') 7>&-; printf ' 3:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: find (dirname (script)) -maxdepth 1 -type f -name *.deb -...' 'pipeline stage 2 failed: sort -z' 'pipeline stage 3 failed: xargs "-0" -r sha256sum' || exit "$?"
