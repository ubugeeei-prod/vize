#!/bin/sh
# Generated from '../check-the-round-trip-theorem-and-each-emitted-value.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../check-the-round-trip-theorem-and-each-emitted-value.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../check-the-round-trip-theorem-and-each-emitted-value.ush' --checked-in -o 'check-the-round-trip-theorem-and-each-emitted-value.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'check-the-round-trip-theorem-and-each-emitted-value.ush'
__ush_origin='../check-the-round-trip-theorem-and-each-emitted-value.ush'

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
  -h|--help) printf '%s' 'check-the-round-trip-theorem-and-each-emitted-value.ush

Usage:
  check-the-round-trip-theorem-and-each-emitted-value.ush [--help]
  check-the-round-trip-theorem-and-each-emitted-value.ush [--man [ITEM]]
  check-the-round-trip-theorem-and-each-emitted-value.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
check-the-round-trip-theorem-and-each-emitted-value.ush

SYNOPSIS
check-the-round-trip-theorem-and-each-emitted-value.ush [--help]
check-the-round-trip-theorem-and-each-emitted-value.ush [--man [ITEM]]
check-the-round-trip-theorem-and-each-emitted-value.ush [--complete [PREFIX]]
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

ush_fn_std__fs__write_text() {
  __ush_path=$(ush_fn_std__path__resolve "$1")
  printf '%s' "$2" > "$__ush_path"
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

__ush_at 8 'let temp = $env::RUNNER_TEMP'
__ush_env_0=${RUNNER_TEMP-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_8_temp="${__ush_env_0}"
# shellcheck disable=SC2034
temp="${__ush_binding_8_temp}"
__ush_at 9 'let archive = $"$temp/lean.tar.zst"'
__ush_binding_9_archive="${__ush_binding_8_temp}"'/lean.tar.zst'
# shellcheck disable=SC2034
archive="${__ush_binding_9_archive}"
__ush_at 10 'let lean = $"$temp/lean-4.33.1-linux/bin/lean"'
__ush_binding_10_lean="${__ush_binding_8_temp}"'/lean-4.33.1-linux/bin/lean'
# shellcheck disable=SC2034
lean="${__ush_binding_10_lean}"
__ush_at 11 'let axioms = $"$temp/literal-axioms"'
__ush_binding_11_axioms="${__ush_binding_8_temp}"'/literal-axioms'
# shellcheck disable=SC2034
axioms="${__ush_binding_11_axioms}"
__ush_at 12 'let emitted_rows = $"$temp/rust-literals.jsonl"'
__ush_binding_12_emitted_rows="${__ush_binding_8_temp}"'/rust-literals.jsonl'
# shellcheck disable=SC2034
emitted_rows="${__ush_binding_12_emitted_rows}"
__ush_at 13 'let wrong_rows = $"$temp/wrong-literal.jsonl"'
__ush_binding_13_wrong_rows="${__ush_binding_8_temp}"'/wrong-literal.jsonl'
# shellcheck disable=SC2034
wrong_rows="${__ush_binding_13_wrong_rows}"
__ush_at 14 'let rejection = $"$temp/literal-rejection"'
__ush_binding_14_rejection="${__ush_binding_8_temp}"'/literal-rejection'
# shellcheck disable=SC2034
rejection="${__ush_binding_14_rejection}"
__ush_at 16 'curl -fsSL --retry 3 -o $archive https://github.com/leanprover/lean4/releases/download/v4.33.1/lean-4.33.1-linux.tar.zst'
command curl '-fsSL' '--retry' 3 '-o' "${__ush_binding_9_archive}" 'https://github.com/leanprover/lean4/releases/download/v4.33.1/lean-4.33.1-linux.tar.zst'
__ush_at 17 '$"890afd185370f85666025b883914ab4f4b339136f8c96167b69cfb62aecaf235  $archive\n" | sha256sum --check --strict'
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (printf '%s' '890afd185370f85666025b883914ab4f4b339136f8c96167b69cfb62aecaf235  '"${__ush_binding_9_archive}""${__ush_lf}") 7>&-; printf ' 1:%s' "$?" >&7; } |
  { (command sha256sum '--check' '--strict') 7>&-; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: $"890afd185370f85666025b883914ab4f4b339136f8c96167b69cfb6...' 'pipeline stage 2 failed: sha256sum --check --strict' || exit "$?"
__ush_at 18 'tar --zstd -xf $archive -C $temp'
command tar '--zstd' '-xf' "${__ush_binding_9_archive}" '-C' "${__ush_binding_8_temp}"
__ush_at 19 'exec $lean ["-DwarningAsError=true", "Emission.lean"] | tee $axioms'
__ush_match_1__len='2'
# shellcheck disable=SC2034
__ush_match_1__0='-DwarningAsError=true'
# shellcheck disable=SC2034
__ush_match_1__1='Emission.lean'
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (ush_fn_std__process__exec "${__ush_binding_10_lean}" "$(ush_fn_std__pack __ush_match_1)") 7>&-; printf ' 1:%s' "$?" >&7; } |
  { (command tee "${__ush_binding_11_axioms}") 7>&-; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: exec $lean ["-DwarningAsError=true", "Emission.lean"]' 'pipeline stage 2 failed: tee $axioms' || exit "$?"
__ush_at 20 'grep -Fx "'"'"'Davinci.StringLiteral.literal_round_trip'"'"' depends on axioms: [propext, Quot.sound]" $axioms'
command grep '-Fx' ''"'"'Davinci.StringLiteral.literal_round_trip'"'"' depends on axioms: [propext, Quot.sound]' "${__ush_binding_11_axioms}"
__ush_at 21 'exec $lean ["-DwarningAsError=true", "--run", "Emission.lean", emitted_rows]'
__ush_match_3__len='4'
# shellcheck disable=SC2034
__ush_match_3__0='-DwarningAsError=true'
# shellcheck disable=SC2034
__ush_match_3__1='--run'
# shellcheck disable=SC2034
__ush_match_3__2='Emission.lean'
# shellcheck disable=SC2034
__ush_match_3__3="${__ush_binding_12_emitted_rows}"
( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__process__exec "${__ush_binding_10_lean}" "$(ush_fn_std__pack __ush_match_3)" )
__ush_at 22 'node (from_source "check-emitted-literal-values.mts") $emitted_rows'
__ush_value_5="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__from_source 'check-emitted-literal-values.mts')" || exit "$?"
command node "${__ush_value_5}" "${__ush_binding_12_emitted_rows}"
__ush_at 25 'write_text $wrong_rows ('"'"'["x","\"y\""]'"'"' + "\n")'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__write_text "${__ush_binding_13_wrong_rows}" '["x","\"y\""]'''"${__ush_lf}"'' )
__ush_at 26 'if exec $lean ["--run", "Emission.lean", wrong_rows] |& to_file $rejection {'
if {
__ush_match_6__len='3'
# shellcheck disable=SC2034
__ush_match_6__0='--run'
# shellcheck disable=SC2034
__ush_match_6__1='Emission.lean'
# shellcheck disable=SC2034
__ush_match_6__2="${__ush_binding_13_wrong_rows}"
__ush_std_fs_path "${__ush_binding_14_rejection}" || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
__ush_path_8=$__ush_path
(ush_fn_std__process__exec "${__ush_binding_10_lean}" "$(ush_fn_std__pack __ush_match_6)") >"$__ush_path_8" 2>&1
}; then
  __ush_at 27 'fail 1 "The round-trip check accepted a different emitted spelling"'
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'The round-trip check accepted a different emitted spelling'
  exit "$__ush_status"
fi
__ush_at 29 'grep -F "different emitted spelling at row 1" $rejection'
command grep '-F' 'different emitted spelling at row 1' "${__ush_binding_14_rejection}"
