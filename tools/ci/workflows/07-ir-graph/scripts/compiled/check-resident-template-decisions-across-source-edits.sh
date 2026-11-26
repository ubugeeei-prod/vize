#!/bin/sh
# Generated from '../check-resident-template-decisions-across-source-edits.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../check-resident-template-decisions-across-source-edits.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../check-resident-template-decisions-across-source-edits.ush' --checked-in -o 'check-resident-template-decisions-across-source-edits.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'check-resident-template-decisions-across-source-edits.ush'
__ush_origin='../check-resident-template-decisions-across-source-edits.ush'

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
  -h|--help) printf '%s' 'check-resident-template-decisions-across-source-edits.ush

Usage:
  check-resident-template-decisions-across-source-edits.ush [--help]
  check-resident-template-decisions-across-source-edits.ush [--man [ITEM]]
  check-resident-template-decisions-across-source-edits.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
check-resident-template-decisions-across-source-edits.ush

SYNOPSIS
check-resident-template-decisions-across-source-edits.ush [--help]
check-resident-template-decisions-across-source-edits.ush [--man [ITEM]]
check-resident-template-decisions-across-source-edits.ush [--complete [PREFIX]]
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

# raises: unknown
ush_fn_tlc() {
  __ush_fn_tlc_arg_0="${1}"
  # shellcheck disable=SC2034
  jar="${__ush_fn_tlc_arg_0}"
  __ush_fn_tlc_arg_1="${2}"
  # shellcheck disable=SC2034
  metadir="${__ush_fn_tlc_arg_1}"
  __ush_fn_tlc_arg_2="${3}"
  # shellcheck disable=SC2034
  config="${__ush_fn_tlc_arg_2}"
  __ush_at 8 'java -cp $jar tlc2.TLC -workers 1 -metadir $metadir -config $config TemplateRevision.tla'
  command java '-cp' "${__ush_fn_tlc_arg_0}" 'tlc2.TLC' '-workers' 1 '-metadir' "${__ush_fn_tlc_arg_1}" '-config' "${__ush_fn_tlc_arg_2}" 'TemplateRevision.tla' || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
}

__ush_at 11 'let temp = $env::RUNNER_TEMP'
__ush_env_0=${RUNNER_TEMP-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_11_temp="${__ush_env_0}"
# shellcheck disable=SC2034
temp="${__ush_binding_11_temp}"
__ush_at 12 'let lean = $"$temp/lean-4.33.1-linux/bin/lean"'
__ush_binding_12_lean="${__ush_binding_11_temp}"'/lean-4.33.1-linux/bin/lean'
# shellcheck disable=SC2034
lean="${__ush_binding_12_lean}"
__ush_at 13 'let jar = $"$temp/tlc.jar"'
__ush_binding_13_jar="${__ush_binding_11_temp}"'/tlc.jar'
# shellcheck disable=SC2034
jar="${__ush_binding_13_jar}"
__ush_at 14 'let projection = $"$temp/template-projection"'
__ush_binding_14_projection="${__ush_binding_11_temp}"'/template-projection'
# shellcheck disable=SC2034
projection="${__ush_binding_14_projection}"
__ush_at 16 'exec $lean ["-DwarningAsError=true", "TemplateProjection.lean"] | tee $projection'
__ush_match_1__len='2'
# shellcheck disable=SC2034
__ush_match_1__0='-DwarningAsError=true'
# shellcheck disable=SC2034
__ush_match_1__1='TemplateProjection.lean'
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (ush_fn_std__process__exec "${__ush_binding_12_lean}" "$(ush_fn_std__pack __ush_match_1)") 7>&-; printf ' 1:%s' "$?" >&7; } |
  { (command tee "${__ush_binding_14_projection}") 7>&-; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: exec $lean ["-DwarningAsError=true", "TemplateProjection....' 'pipeline stage 2 failed: tee $projection' || exit "$?"
__ush_at 17 'grep -Fx "'"'"'Davinci.TemplateProjection.exact'"'"' depends on axioms: [propext]" $projection'
command grep '-Fx' ''"'"'Davinci.TemplateProjection.exact'"'"' depends on axioms: [propext]' "${__ush_binding_14_projection}"
__ush_at 18 'grep -Fx "'"'"'Davinci.TemplateProjection.endpoints_fit'"'"' does not depend on any axioms" $projection'
command grep '-Fx' ''"'"'Davinci.TemplateProjection.endpoints_fit'"'"' does not depend on any axioms' "${__ush_binding_14_projection}"
__ush_at 19 'tlc $jar $"$temp/template-revision" TemplateRevision.cfg'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_tlc "${__ush_binding_13_jar}" "${__ush_binding_11_temp}"'/template-revision' 'TemplateRevision.cfg' )
__ush_at 20 'for defect in ["Content", "Base", "Duplicate"] {'
__ush_match_3__len='3'
# shellcheck disable=SC2034
__ush_match_3__0='Content'
# shellcheck disable=SC2034
__ush_match_3__1='Base'
# shellcheck disable=SC2034
__ush_match_3__2='Duplicate'
__ush_for_list_index_4=0
while [ "${__ush_for_list_index_4}" -lt "${__ush_match_3__len}" ]; do
  __ush_for_value_5=''
  eval "__ush_for_value_5=\"\${__ush_match_3__${__ush_for_list_index_4}}\""
  defect="${__ush_for_value_5}"
  __ush_for_list_index_4=$((__ush_for_list_index_4 + 1))
  __ush_at 21 'let log = $"$temp/template-$defect.log"'
  __ush_binding_21_log="${__ush_binding_11_temp}"'/template-'"${defect}"'.log'
  # shellcheck disable=SC2034
  log="${__ush_binding_21_log}"
  __ush_at 23 'let result = status_of tlc $jar $"$temp/template-$defect" $"TemplateRevision$defect.cfg" |& to_file $log'
  __ush_std_fs_path "${__ush_binding_21_log}" || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
  __ush_path_6=$__ush_path
  if ( __ush_capture_return='0' __ush_return_path=''; ush_fn_tlc "${__ush_binding_13_jar}" "${__ush_binding_11_temp}"'/template-'"${defect}" 'TemplateRevision'"${defect}"'.cfg' ) >"$__ush_path_6" 2>&1; then __ush_status_7=0; else __ush_status_7=$?; fi
  __ush_binding_23_result="${__ush_status_7}"
  # shellcheck disable=SC2034
  result="${__ush_binding_23_result}"
  __ush_at 24 'if result != 12 {'
  if {
  [ "$(if [ "${__ush_binding_23_result}" -ne 12 ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
  }; then
    __ush_at 25 'fail 1 $"TLC exited $result, not 12, on the $defect defect"'
    __ush_status=1
    if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
    __ush_report "$__ush_status" 'TLC exited '"${__ush_binding_23_result}"', not 12, on the '"${defect}"' defect'
    exit "$__ush_status"
  fi
  __ush_at 27 'grep -F "Invariant ExactReport is violated" $log'
  command grep '-F' 'Invariant ExactReport is violated' "${__ush_binding_21_log}"
done
