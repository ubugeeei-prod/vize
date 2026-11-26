#!/bin/sh
# Generated from '../verify-source-coordinates-and-style-headers.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../verify-source-coordinates-and-style-headers.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../verify-source-coordinates-and-style-headers.ush' --checked-in -o 'verify-source-coordinates-and-style-headers.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'verify-source-coordinates-and-style-headers.ush'
__ush_origin='../verify-source-coordinates-and-style-headers.ush'

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
  -h|--help) printf '%s' 'verify-source-coordinates-and-style-headers.ush

Usage:
  verify-source-coordinates-and-style-headers.ush [--help]
  verify-source-coordinates-and-style-headers.ush [--man [ITEM]]
  verify-source-coordinates-and-style-headers.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
verify-source-coordinates-and-style-headers.ush

SYNOPSIS
verify-source-coordinates-and-style-headers.ush [--help]
verify-source-coordinates-and-style-headers.ush [--man [ITEM]]
verify-source-coordinates-and-style-headers.ush [--complete [PREFIX]]
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

__ush_at 5 'let lean_version = "4.33.1"'
__ush_binding_5_lean_version='4.33.1'
# shellcheck disable=SC2034
lean_version="${__ush_binding_5_lean_version}"
__ush_at 6 'let lean = $"lean-$lean_version-linux/bin/lean"'
__ush_binding_6_lean='lean-'"${__ush_binding_5_lean_version}"'-linux/bin/lean'
# shellcheck disable=SC2034
lean="${__ush_binding_6_lean}"
__ush_at 7 'let formal = $"$env::GITHUB_WORKSPACE/tests/formal"'
__ush_env_0=${GITHUB_WORKSPACE-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable GITHUB_WORKSPACE is not set or is empty'; exit 1; fi
__ush_binding_7_formal="${__ush_env_0}"'/tests/formal'
# shellcheck disable=SC2034
formal="${__ush_binding_7_formal}"
__ush_at 9 'cd $env::RUNNER_TEMP {'
__ush_env_1=${RUNNER_TEMP-}
if [ -z "${__ush_env_1}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_cwd_2=$PWD
ush_fn_std__process__chdir "${__ush_env_1}" || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
  __ush_at 10 'curl -fsSL --retry 3 -o lean.tar.zst $"https://github.com/leanprover/lean4/releases/download/v$lean_version/lean-$lean_version-linux.tar.zst"'
  command curl '-fsSL' '--retry' 3 '-o' 'lean.tar.zst' 'https://github.com/leanprover/lean4/releases/download/v'"${__ush_binding_5_lean_version}"'/lean-'"${__ush_binding_5_lean_version}"'-linux.tar.zst'
  __ush_at 11 '"890afd185370f85666025b883914ab4f4b339136f8c96167b69cfb62aecaf235  lean.tar.zst\n" | sha256sum -c'
  { __ush_pipe=$(
    exec 7>&1 >&8 8>&-
    set +e
    { (printf '%s' '890afd185370f85666025b883914ab4f4b339136f8c96167b69cfb62aecaf235  lean.tar.zst'"${__ush_lf}"'') 7>&-; printf ' 1:%s' "$?" >&7; } |
    { (command sha256sum '-c') 7>&-; printf ' 2:%s' "$?" >&7; }
  ); } 8>&1
  __ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: "890afd185370f85666025b883914ab4f4b339136f8c96167b69cfb62...' 'pipeline stage 2 failed: sha256sum -c' || exit "$?"
  __ush_at 12 'tar --zstd -xf lean.tar.zst'
  command tar '--zstd' '-xf' 'lean.tar.zst'
  __ush_at 13 'exec $lean ["-DwarningAsError=true", $"$formal/source-map/LineBreaks.lean"] | tee line-breaks.log'
  __ush_match_3__len='2'
  # shellcheck disable=SC2034
  __ush_match_3__0='-DwarningAsError=true'
  # shellcheck disable=SC2034
  __ush_match_3__1="${__ush_binding_7_formal}"'/source-map/LineBreaks.lean'
  { __ush_pipe=$(
    exec 7>&1 >&8 8>&-
    set +e
    { (ush_fn_std__process__exec "${__ush_binding_6_lean}" "$(ush_fn_std__pack __ush_match_3)") 7>&-; printf ' 1:%s' "$?" >&7; } |
    { (command tee 'line-breaks.log') 7>&-; printf ' 2:%s' "$?" >&7; }
  ); } 8>&1
  __ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: exec $lean ["-DwarningAsError=true", $"$formal/source-map...' 'pipeline stage 2 failed: tee line-breaks.log' || exit "$?"
  __ush_at 14 'grep -Fx "'"'"'Davinci.SourceMap.LineBreaks.first_exact'"'"' does not depend on any axioms" line-breaks.log'
  command grep '-Fx' ''"'"'Davinci.SourceMap.LineBreaks.first_exact'"'"' does not depend on any axioms' 'line-breaks.log'
  __ush_at 15 'grep -Fx "'"'"'Davinci.SourceMap.LineBreaks.last_exact'"'"' depends on axioms: [propext]" line-breaks.log'
  command grep '-Fx' ''"'"'Davinci.SourceMap.LineBreaks.last_exact'"'"' depends on axioms: [propext]' 'line-breaks.log'
  __ush_at 16 'exec $lean ["-DwarningAsError=true", $"$formal/source-map/Coordinates.lean"] | tee theorem.log'
  __ush_match_5__len='2'
  # shellcheck disable=SC2034
  __ush_match_5__0='-DwarningAsError=true'
  # shellcheck disable=SC2034
  __ush_match_5__1="${__ush_binding_7_formal}"'/source-map/Coordinates.lean'
  { __ush_pipe=$(
    exec 7>&1 >&8 8>&-
    set +e
    { (ush_fn_std__process__exec "${__ush_binding_6_lean}" "$(ush_fn_std__pack __ush_match_5)") 7>&-; printf ' 1:%s' "$?" >&7; } |
    { (command tee 'theorem.log') 7>&-; printf ' 2:%s' "$?" >&7; }
  ); } 8>&1
  __ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: exec $lean ["-DwarningAsError=true", $"$formal/source-map...' 'pipeline stage 2 failed: tee theorem.log' || exit "$?"
  __ush_at 17 'grep -Fx "'"'"'Davinci.SourceMap.column_exact'"'"' depends on axioms: [propext, Quot.sound]" theorem.log'
  command grep '-Fx' ''"'"'Davinci.SourceMap.column_exact'"'"' depends on axioms: [propext, Quot.sound]' 'theorem.log'
  __ush_at 18 'grep -Fx "'"'"'Davinci.SourceMap.diagnostic_range_rebased'"'"' depends on axioms: [propext, Quot.sound]" theorem.log'
  command grep '-Fx' ''"'"'Davinci.SourceMap.diagnostic_range_rebased'"'"' depends on axioms: [propext, Quot.sound]' 'theorem.log'
  __ush_at 19 'exec $lean ["-DwarningAsError=true", $"$formal/style-format/Header.lean"] | tee header.log'
  __ush_match_7__len='2'
  # shellcheck disable=SC2034
  __ush_match_7__0='-DwarningAsError=true'
  # shellcheck disable=SC2034
  __ush_match_7__1="${__ush_binding_7_formal}"'/style-format/Header.lean'
  { __ush_pipe=$(
    exec 7>&1 >&8 8>&-
    set +e
    { (ush_fn_std__process__exec "${__ush_binding_6_lean}" "$(ush_fn_std__pack __ush_match_7)") 7>&-; printf ' 1:%s' "$?" >&7; } |
    { (command tee 'header.log') 7>&-; printf ' 2:%s' "$?" >&7; }
  ); } 8>&1
  __ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: exec $lean ["-DwarningAsError=true", $"$formal/style-form...' 'pipeline stage 2 failed: tee header.log' || exit "$?"
  __ush_at 20 'grep -Fx "'"'"'Davinci.StyleFormat.accepted_iff'"'"' depends on axioms: [propext, Classical.choice, Quot.sound]" header.log'
  command grep '-Fx' ''"'"'Davinci.StyleFormat.accepted_iff'"'"' depends on axioms: [propext, Classical.choice, Quot.sound]' 'header.log'
cd "$__ush_cwd_2" || { __ush_status=$?; __ush_report "$__ush_status" 'cannot return to the directory the cd block started in'; exit "$__ush_status"; }
