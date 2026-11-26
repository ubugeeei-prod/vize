#!/bin/sh
# Generated from '../model-native-diagnostic-publication.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../model-native-diagnostic-publication.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../model-native-diagnostic-publication.ush' --checked-in -o 'model-native-diagnostic-publication.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'model-native-diagnostic-publication.ush'
__ush_origin='../model-native-diagnostic-publication.ush'

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
  -h|--help) printf '%s' 'model-native-diagnostic-publication.ush

Usage:
  model-native-diagnostic-publication.ush [--help]
  model-native-diagnostic-publication.ush [--man [ITEM]]
  model-native-diagnostic-publication.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
model-native-diagnostic-publication.ush

SYNOPSIS
model-native-diagnostic-publication.ush [--help]
model-native-diagnostic-publication.ush [--man [ITEM]]
model-native-diagnostic-publication.ush [--complete [PREFIX]]
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
  __ush_at 8 'java -cp $jar tlc2.TLC -workers 1 -metadir $metadir -config $config Publication.tla'
  command java '-cp' "${__ush_fn_tlc_arg_0}" 'tlc2.TLC' '-workers' 1 '-metadir' "${__ush_fn_tlc_arg_1}" '-config' "${__ush_fn_tlc_arg_2}" 'Publication.tla' || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
}

__ush_at 11 'let temp = $env::RUNNER_TEMP'
__ush_env_0=${RUNNER_TEMP-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_11_temp="${__ush_env_0}"
# shellcheck disable=SC2034
temp="${__ush_binding_11_temp}"
__ush_at 12 'let jar = $"$temp/ambient-tlc.jar"'
__ush_binding_12_jar="${__ush_binding_11_temp}"'/ambient-tlc.jar'
# shellcheck disable=SC2034
jar="${__ush_binding_12_jar}"
__ush_at 14 'curl -fsSL --retry 3 -o $jar https://github.com/tlaplus/tlaplus/releases/download/v1.7.4/tla2tools.jar'
command curl '-fsSL' '--retry' 3 '-o' "${__ush_binding_12_jar}" 'https://github.com/tlaplus/tlaplus/releases/download/v1.7.4/tla2tools.jar'
__ush_at 15 '$"936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88  $jar\n" | sha256sum -c -'
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (printf '%s' '936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88  '"${__ush_binding_12_jar}""${__ush_lf}") 7>&-; printf ' 1:%s' "$?" >&7; } |
  { (command sha256sum '-c' '-') 7>&-; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: $"936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d...' 'pipeline stage 2 failed: sha256sum -c -' || exit "$?"
__ush_at 16 'tlc $jar $"$temp/ambient-model" Publication.cfg'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_tlc "${__ush_binding_12_jar}" "${__ush_binding_11_temp}"'/ambient-model' 'Publication.cfg' )
__ush_at 18 'for fault in ["ack", "native", "doc", "sources", "host", "revision", "publish"] {'
__ush_match_1__len='7'
# shellcheck disable=SC2034
__ush_match_1__0='ack'
# shellcheck disable=SC2034
__ush_match_1__1='native'
# shellcheck disable=SC2034
__ush_match_1__2='doc'
# shellcheck disable=SC2034
__ush_match_1__3='sources'
# shellcheck disable=SC2034
__ush_match_1__4='host'
# shellcheck disable=SC2034
__ush_match_1__5='revision'
# shellcheck disable=SC2034
__ush_match_1__6='publish'
__ush_for_list_index_2=0
while [ "${__ush_for_list_index_2}" -lt "${__ush_match_1__len}" ]; do
  __ush_for_value_3=''
  eval "__ush_for_value_3=\"\${__ush_match_1__${__ush_for_list_index_2}}\""
  fault="${__ush_for_value_3}"
  __ush_for_list_index_2=$((__ush_for_list_index_2 + 1))
  __ush_at 19 'let config = $"$temp/ambient-$fault.cfg"'
  __ush_binding_19_config="${__ush_binding_11_temp}"'/ambient-'"${fault}"'.cfg'
  # shellcheck disable=SC2034
  config="${__ush_binding_19_config}"
  __ush_at 20 'let log = $"$temp/ambient-$fault.log"'
  __ush_binding_20_log="${__ush_binding_11_temp}"'/ambient-'"${fault}"'.log'
  # shellcheck disable=SC2034
  log="${__ush_binding_20_log}"
  __ush_at 21 'awk -v fault=$fault -f (from_source "ambient-publication-fault.awk") Publication.cfg | to_file $config'
  __ush_value_4="${fault}"
  __ush_value_5="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__from_source 'ambient-publication-fault.awk')" || exit "$?"
  __ush_std_fs_path "${__ush_binding_19_config}" || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
  __ush_path_6=$__ush_path
  { __ush_pipe=$(
    exec 7>&1 >&8 8>&-
    set +e
    { (command awk '-v' 'fault='"${__ush_value_4}" '-f' "${__ush_value_5}" 'Publication.cfg') 7>&- >"$__ush_path_6"; printf ' 1:%s' "$?" >&7; }
  ); } 8>&1
  __ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: awk -v fault=$fault -f (from_source "ambient-publication-...' || exit "$?"
  __ush_at 23 'let result = status_of tlc $jar $"$temp/ambient-$fault" $config |& to_file $log'
  __ush_std_fs_path "${__ush_binding_20_log}" || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
  __ush_path_7=$__ush_path
  if ( __ush_capture_return='0' __ush_return_path=''; ush_fn_tlc "${__ush_binding_12_jar}" "${__ush_binding_11_temp}"'/ambient-'"${fault}" "${__ush_binding_19_config}" ) >"$__ush_path_7" 2>&1; then __ush_status_8=0; else __ush_status_8=$?; fi
  __ush_binding_23_result="${__ush_status_8}"
  # shellcheck disable=SC2034
  result="${__ush_binding_23_result}"
  __ush_at 24 'if result != 12 {'
  if {
  [ "$(if [ "${__ush_binding_23_result}" -ne 12 ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
  }; then
    __ush_at 25 'fail 1 $"TLC exited $result, not 12, with the $fault fault injected"'
    __ush_status=1
    if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
    __ush_report "$__ush_status" 'TLC exited '"${__ush_binding_23_result}"', not 12, with the '"${fault}"' fault injected'
    exit "$__ush_status"
  fi
  __ush_at 27 'grep -F "Invariant ExactPublication is violated" $log'
  command grep '-F' 'Invariant ExactPublication is violated' "${__ush_binding_20_log}"
done
