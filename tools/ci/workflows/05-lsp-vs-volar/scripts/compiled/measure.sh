#!/bin/sh
# Generated from '../measure.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../measure.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../measure.ush' --checked-in -o 'measure.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'measure.ush'
__ush_origin='../measure.ush'

__ush_at() { __ush_line=$1; __ush_source=$2; __ush_file=${3:-$__ush_origin}; }
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
    printf '%s:%s: %s (exit %s)\n  %s\n' "${__ush_file:-$__ush_origin}" "$__ush_line" "$__ush_message" "$1" "$__ush_source" >&2
    __ush_line=''
  fi
}
trap '__ush_report "$?"' 0

case "${1:-}" in
  -h|--help) printf '%s' 'measure.ush

Usage:
  measure.ush [--help]
  measure.ush [--man [ITEM]]
  measure.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
measure.ush

SYNOPSIS
measure.ush [--help]
measure.ush [--man [ITEM]]
measure.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

exec 3>&1

__ush_std_env_validate() {
  case "$1" in
    ''|[0-9]*|*[!A-Za-z0-9_]*)
      printf '%s\n' "ush std::env: invalid env name: $1" >&2
      return 1
      ;;
  esac
}

ush_fn_std__env__get() {
  __ush_std_env_validate "$1" || return "$?"
  eval "printf '%s' \"\${$1-}\""
}

ush_fn_std__fs__read_text() {
  __ush_path=$(ush_fn_std__path__resolve "$1")
  cat "$__ush_path"
}

ush_fn_std__fs__write_text() {
  __ush_path=$(ush_fn_std__path__resolve "$1")
  printf '%s' "$2" > "$__ush_path"
}

ush_fn_std__fs__append_text() {
  __ush_path=$(ush_fn_std__path__resolve "$1")
  printf '%s' "$2" >> "$__ush_path"
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

ush_fn_std____extern__int() {
  __ush_sign=''
  __ush_limit=9223372036854775807
  case $1 in
    -*) __ush_sign=-; __ush_digits=${1#-}; __ush_limit=9223372036854775808 ;;
    *) __ush_digits=$1 ;;
  esac
  case $__ush_digits in
    ''|*[!0-9]*) printf '%s\n' 'extern returned an invalid Int' >&2; return 1 ;;
  esac
  while [ "${__ush_digits#0}" != "$__ush_digits" ]; do __ush_digits=${__ush_digits#0}; done
  if [ -z "$__ush_digits" ]; then printf 0; return 0; fi
  if [ "${#__ush_digits}" -gt 19 ] || { [ "${#__ush_digits}" -eq 19 ] && [ "$(printf '%s\n%s\n' "$__ush_digits" "$__ush_limit" | LC_ALL=C sort | tail -n 1)" != "$__ush_limit" ]; }; then
    printf '%s\n' 'extern Int is outside the signed 64-bit range' >&2
    return 1
  fi
  printf '%s%s' "$__ush_sign" "$__ush_digits"
}

__ush_std_encode_args() {
  for __ush_arg do
    printf x
    printf '%s' "$__ush_arg" | od -A n -v -t x1 | tr -d ' \n'
    printf ' '
  done
}

ush_fn_std__run_words() {
  # Run a program whose arguments are tagged words: `s value` is one argument,
  # `l packed` gives every element of a List an argument of its own.
  __ush_program=$1
  shift
  __ush_argv__len=0
  while [ "$#" -gt 0 ]; do
    case $1 in
      s)
        eval "__ush_argv__${__ush_argv__len}=\$2"
        __ush_argv__len=$((__ush_argv__len + 1))
        ;;
      l)
        __ush_words__len=0
        ush_fn_std__unpack __ush_words "$2" || return "$?"
        __ush_index=0
        while [ "$__ush_index" -lt "$__ush_words__len" ]; do
          eval "__ush_argv__${__ush_argv__len}=\${__ush_words__${__ush_index}}"
          __ush_argv__len=$((__ush_argv__len + 1))
          __ush_index=$((__ush_index + 1))
        done
        ;;
    esac
    shift 2
  done
  set --
  __ush_index=0
  while [ "$__ush_index" -lt "$__ush_argv__len" ]; do
    eval "__ush_arg=\${__ush_argv__${__ush_index}}"
    set -- "$@" "$__ush_arg"
    __ush_index=$((__ush_index + 1))
  done
  command "$__ush_program" "$@"
}

ush_fn_std__string__to_int() {
  if __ush_integer=$(ush_fn_std____extern__int "$1" 2>/dev/null); then
    printf '%s' "$__ush_integer"
  else
    printf '%s\n' 'std::string::to_int: expected a signed 64-bit decimal integer' >&2
    return 1
  fi
}

ush_fn_std__string__words() {
  set -f
  IFS=' 	
'
  # shellcheck disable=SC2086
  __ush_std_encode_args $1
}

__ush_std_fs_path() {
  __ush_path=$(ush_fn_std__path__resolve "$1" && printf .) || return "$?"
  __ush_path=${__ush_path%.}
  [ -n "$__ush_path" ] || { printf '%s\n' 'std::fs: path must not be empty' >&2; return 1; }
  case $__ush_path in .|..|/*|./*|../*) ;; *) __ush_path=./$__ush_path ;; esac
}

ush_fn_std__fs__read_lines() {
  __ush_std_fs_path "$1" || return "$?"
  ush_fn_std__io__read_lines < "$__ush_path"
}

ush_fn_std__io__read_lines() {
  __ush_line=''
  while IFS= read -r __ush_line || [ -n "$__ush_line" ]; do
    __ush_std_encode_args "$__ush_line"
  done
}

__ush_std_list_contains() {
  eval "__ush_list_len=\${${1}__len}"
  __ush_list_index=0
  while [ "$__ush_list_index" -lt "$__ush_list_len" ]; do
    eval "__ush_list_item=\${${1}__${__ush_list_index}}"
    if [ "$__ush_list_item" = "$2" ]; then return 0; fi
    __ush_list_index=$((__ush_list_index + 1))
  done
  return 1
}

# raises: unknown
ush_fn_github__summary() {
  __ush_fn_github__summary_arg_0="${1}"
  # shellcheck disable=SC2034
  markdown="${__ush_fn_github__summary_arg_0}"
  __ush_at 35 'append_text $env::GITHUB_STEP_SUMMARY $"$markdown\n"' '../../../../lib/github.ush'
  __ush_env_0=${GITHUB_STEP_SUMMARY-}
  if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable GITHUB_STEP_SUMMARY is not set or is empty'; return 1; fi
  __ush_value_1="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__fs__append_text "${__ush_env_0}" "${__ush_fn_github__summary_arg_0}""${__ush_lf}")" || return "$?"
  if [ -n "${__ush_return_path:-}" ]; then
    printf '%s' '' > "$__ush_return_path"
  elif [ "${__ush_capture_return:-0}" = '1' ]; then
    printf '%s' '' >&4
  fi
  return 0
}

# raises: unknown
ush_fn_version_metas() {
  __ush_fn_version_metas_arg_0="${1}"
  # shellcheck disable=SC2034
  versions="${__ush_fn_version_metas_arg_0}"
  __ush_at 15 '[argument for line in read_lines $versions for argument in ["--meta", version_meta $line]]'
  __ush_comprehension_count_3=0
  __ush_packed_5="$(__ush_return_path='' __ush_capture_return=1 ush_fn_std__fs__read_lines "${__ush_fn_version_metas_arg_0}")" || return "$?"
  __ush_match_4__len=0
  ush_fn_std__unpack __ush_match_4 "${__ush_packed_5}"
  __ush_comprehension_index_6=0
  while [ "${__ush_comprehension_index_6}" -lt "${__ush_match_4__len}" ]; do
  eval "__ush_comprehension_item_7=\"\${__ush_match_4__${__ush_comprehension_index_6}}\""
  __ush_match_8__len='2'
  # shellcheck disable=SC2034
  __ush_match_8__0='--meta'
  __ush_value_9="$(__ush_return_path='' __ush_capture_return='1' ush_fn_version_meta "${__ush_comprehension_item_7}" 4>&1 >&3)" || return "$?"
  # shellcheck disable=SC2034
  __ush_match_8__1="${__ush_value_9}"
  __ush_comprehension_index_10=0
  while [ "${__ush_comprehension_index_10}" -lt "${__ush_match_8__len}" ]; do
  eval "__ush_comprehension_item_11=\"\${__ush_match_8__${__ush_comprehension_index_10}}\""
  # shellcheck disable=SC2034
  __ush_comprehension_value_12="${__ush_comprehension_item_11}"
  eval "__ush_match_2__${__ush_comprehension_count_3}=\$__ush_comprehension_value_12"
  __ush_comprehension_count_3=$((__ush_comprehension_count_3 + 1))
  __ush_comprehension_index_10=$((__ush_comprehension_index_10 + 1))
  done
  __ush_comprehension_index_6=$((__ush_comprehension_index_6 + 1))
  done
  __ush_match_2__len="${__ush_comprehension_count_3}"
  if [ -n "${__ush_return_path:-}" ]; then
    printf '%s' "$(ush_fn_std__pack __ush_match_2)" > "$__ush_return_path"
  elif [ "${__ush_capture_return:-0}" = '1' ]; then
    printf '%s' "$(ush_fn_std__pack __ush_match_2)" >&4
  fi
  return 0
}

# raises: unknown
ush_fn_version_meta() {
  __ush_fn_version_meta_arg_0="${1}"
  # shellcheck disable=SC2034
  line="${__ush_fn_version_meta_arg_0}"
  __ush_at 10 'let fields = words $line'
  __ush_packed_13="$(__ush_return_path='' __ush_capture_return=1 ush_fn_std__string__words "${__ush_fn_version_meta_arg_0}")" || return "$?"
  __ush_binding_10_fields__len=0
  ush_fn_std__unpack __ush_binding_10_fields "${__ush_packed_13}"
  __ush_at 11 '$"${fields[0]}=${fields[1]}"'
  __ush_index_14=0
  if [ "${__ush_index_14}" -lt 0 ] || [ "${__ush_index_14}" -ge "${__ush_binding_10_fields__len}" ]; then __ush_report 1 "index ${__ush_index_14} is out of range for a list of length ${__ush_binding_10_fields__len}"; return 1; fi
  __ush_item_15=''
  eval "__ush_item_15=\"\${__ush_binding_10_fields__${__ush_index_14}}\""
  __ush_index_16=1
  if [ "${__ush_index_16}" -lt 0 ] || [ "${__ush_index_16}" -ge "${__ush_binding_10_fields__len}" ]; then __ush_report 1 "index ${__ush_index_16} is out of range for a list of length ${__ush_binding_10_fields__len}"; return 1; fi
  __ush_item_17=''
  eval "__ush_item_17=\"\${__ush_binding_10_fields__${__ush_index_16}}\""
  if [ -n "${__ush_return_path:-}" ]; then
    printf '%s' "${__ush_item_15}"'='"${__ush_item_17}" > "$__ush_return_path"
  elif [ "${__ush_capture_return:-0}" = '1' ]; then
    printf '%s' "${__ush_item_15}"'='"${__ush_item_17}" >&4
  fi
  return 0
}

__ush_at 18 'let temp = $env::RUNNER_TEMP'
__ush_env_18=${RUNNER_TEMP-}
if [ -z "${__ush_env_18}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_18_temp="${__ush_env_18}"
# shellcheck disable=SC2034
temp="${__ush_binding_18_temp}"
__ush_at 19 'let tools = $"$temp/tools/node_modules"'
__ush_binding_19_tools="${__ush_binding_18_temp}"'/tools/node_modules'
# shellcheck disable=SC2034
tools="${__ush_binding_19_tools}"
__ush_at 20 'let job = $env::JOB'
__ush_env_19=${JOB-}
if [ -z "${__ush_env_19}" ]; then __ush_report 1 'environment variable JOB is not set or is empty'; exit 1; fi
__ush_binding_20_job="${__ush_env_19}"
# shellcheck disable=SC2034
job="${__ush_binding_20_job}"
__ush_at 21 'let fixture_ids = words $env::FIXTURE_IDS'
__ush_env_20=${FIXTURE_IDS-}
if [ -z "${__ush_env_20}" ]; then __ush_report 1 'environment variable FIXTURE_IDS is not set or is empty'; exit 1; fi
__ush_packed_21="$(__ush_return_path='' __ush_capture_return=1 ush_fn_std__string__words "${__ush_env_20}")" || exit "$?"
__ush_binding_21_fixture_ids__len=0
ush_fn_std__unpack __ush_binding_21_fixture_ids "${__ush_packed_21}"
__ush_at 22 'let compare_ref = get COMPARE_REF'
__ush_value_22="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__env__get 'COMPARE_REF')" || exit "$?"
__ush_binding_22_compare_ref="${__ush_value_22}"
# shellcheck disable=SC2034
compare_ref="${__ush_binding_22_compare_ref}"
__ush_at 23 'let tsgo_mapper = get TSGO_MAPPER'
__ush_value_23="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__env__get 'TSGO_MAPPER')" || exit "$?"
__ush_binding_23_tsgo_mapper="${__ush_value_23}"
# shellcheck disable=SC2034
tsgo_mapper="${__ush_binding_23_tsgo_mapper}"
__ush_at 25 'let mut rounds = $env::ROUNDS'
__ush_env_24=${ROUNDS-}
if [ -z "${__ush_env_24}" ]; then __ush_report 1 'environment variable ROUNDS is not set or is empty'; exit 1; fi
__ush_binding_25_rounds="${__ush_env_24}"
# shellcheck disable=SC2034
rounds="${__ush_binding_25_rounds}"
__ush_at 26 'let mut cycles = $env::CYCLES'
__ush_env_25=${CYCLES-}
if [ -z "${__ush_env_25}" ]; then __ush_report 1 'environment variable CYCLES is not set or is empty'; exit 1; fi
__ush_binding_26_cycles="${__ush_env_25}"
# shellcheck disable=SC2034
cycles="${__ush_binding_26_cycles}"
__ush_at 29 'if $env::GITHUB_EVENT_NAME == "pull_request" && job == "scale-giant-sfc" {'
if {
{
__ush_env_26=${GITHUB_EVENT_NAME-}
if [ -z "${__ush_env_26}" ]; then __ush_report 1 'environment variable GITHUB_EVENT_NAME is not set or is empty'; exit 1; fi
[ "$(if [ "${__ush_env_26}" = 'pull_request' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
} && {
[ "$(if [ "${__ush_binding_20_job}" = 'scale-giant-sfc' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}
}; then
  __ush_at 30 'rounds = "1"'
  __ush_binding_25_rounds='1'
  # shellcheck disable=SC2034
  rounds="${__ush_binding_25_rounds}"
  __ush_at 31 'cycles = "1"'
  __ush_binding_26_cycles='1'
  # shellcheck disable=SC2034
  cycles="${__ush_binding_26_cycles}"
fi
__ush_at 35 'let tsrs_fast = $"$temp/tsrs-fast"'
__ush_binding_35_tsrs_fast="${__ush_binding_18_temp}"'/tsrs-fast'
# shellcheck disable=SC2034
tsrs_fast="${__ush_binding_35_tsrs_fast}"
__ush_at 36 'write_text $tsrs_fast $"#!/bin/sh\nexport TSRS_FAST=1\nexec \"$tools/.bin/tsrs\" \"\$@\"\n"'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__write_text "${__ush_binding_35_tsrs_fast}" '#!/bin/sh'"${__ush_lf}"'export TSRS_FAST=1'"${__ush_lf}"'exec "'"${__ush_binding_19_tools}"'/.bin/tsrs" "$@"'"${__ush_lf}" )
__ush_at 37 'chmod +x $tsrs_fast'
command chmod '+x' "${__ush_binding_35_tsrs_fast}"
__ush_at 39 'let mut args = ['
__ush_binding_39_args__len='6'
# shellcheck disable=SC2034
__ush_binding_39_args__0='--vize'
# shellcheck disable=SC2034
__ush_binding_39_args__1='head='"${__ush_binding_18_temp}"'/vize-head'
# shellcheck disable=SC2034
__ush_binding_39_args__2='--vize'
# shellcheck disable=SC2034
__ush_binding_39_args__3='head-tsrs-fast='"${__ush_binding_18_temp}"'/vize-head'
# shellcheck disable=SC2034
__ush_binding_39_args__4='--vize-env'
# shellcheck disable=SC2034
__ush_binding_39_args__5='head-tsrs-fast:CORSA_PATH='"${__ush_binding_35_tsrs_fast}"
__ush_at 44 'for id in fixture_ids {'
__ush_match_27__len="${__ush_binding_21_fixture_ids__len}"
__ush_copy_index_28=0
while [ "$__ush_copy_index_28" -lt "${__ush_binding_21_fixture_ids__len}" ]; do
  eval "__ush_copy_value_29=\"\${__ush_binding_21_fixture_ids__$__ush_copy_index_28}\""
  eval "__ush_match_27__${__ush_copy_index_28}=\$__ush_copy_value_29"
  __ush_copy_index_28=$((__ush_copy_index_28 + 1))
done
__ush_for_list_index_30=0
while [ "${__ush_for_list_index_30}" -lt "${__ush_match_27__len}" ]; do
  __ush_for_value_31=''
  eval "__ush_for_value_31=\"\${__ush_match_27__${__ush_for_list_index_30}}\""
  id="${__ush_for_value_31}"
  __ush_for_list_index_30=$((__ush_for_list_index_30 + 1))
  __ush_at 45 'args = args + ["--fixture", id]'
  __ush_match_32__len='2'
  # shellcheck disable=SC2034
  __ush_match_32__0='--fixture'
  # shellcheck disable=SC2034
  __ush_match_32__1="${id}"
  __ush_count_33=${__ush_binding_39_args__len}
  __ush_index_34=0
  while [ "${__ush_index_34}" -lt "${__ush_match_32__len}" ]; do
  eval "__ush_binding_39_args__${__ush_count_33}=\"\${__ush_match_32__${__ush_index_34}}\""
    __ush_index_34=$((__ush_index_34 + 1))
    __ush_count_33=$((__ush_count_33 + 1))
  done
  __ush_binding_39_args__len=${__ush_count_33}
done
__ush_at 47 'if compare_ref != "" {'
if {
[ "$(if [ "${__ush_binding_22_compare_ref}" != '' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 48 'args = args + ['
  __ush_match_35__len='8'
  # shellcheck disable=SC2034
  __ush_match_35__0='--vize'
  # shellcheck disable=SC2034
  __ush_match_35__1='compare='"${__ush_binding_18_temp}"'/vize-compare'
  # shellcheck disable=SC2034
  __ush_match_35__2='--vize'
  # shellcheck disable=SC2034
  __ush_match_35__3='compare-tsrs-fast='"${__ush_binding_18_temp}"'/vize-compare'
  # shellcheck disable=SC2034
  __ush_match_35__4='--vize-env'
  # shellcheck disable=SC2034
  __ush_match_35__5='compare-tsrs-fast:CORSA_PATH='"${__ush_binding_35_tsrs_fast}"
  # shellcheck disable=SC2034
  __ush_match_35__6='--meta'
  # shellcheck disable=SC2034
  __ush_match_35__7='compare='"${__ush_binding_22_compare_ref}"
  __ush_count_36=${__ush_binding_39_args__len}
  __ush_index_37=0
  while [ "${__ush_index_37}" -lt "${__ush_match_35__len}" ]; do
  eval "__ush_binding_39_args__${__ush_count_36}=\"\${__ush_match_35__${__ush_index_37}}\""
    __ush_index_37=$((__ush_index_37 + 1))
    __ush_count_36=$((__ush_count_36 + 1))
  done
  __ush_binding_39_args__len=${__ush_count_36}
fi
__ush_at 55 'if tsgo_mapper != "" {'
if {
[ "$(if [ "${__ush_binding_23_tsgo_mapper}" != '' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 57 'let mapper_modules = $"$temp/mapper/node_modules"'
  __ush_binding_57_mapper_modules="${__ush_binding_18_temp}"'/mapper/node_modules'
  # shellcheck disable=SC2034
  mapper_modules="${__ush_binding_57_mapper_modules}"
  __ush_at 58 'args = args + ["--tsgo-mapper", $"$mapper_modules/@typescript/typescript-linux-x64/lib/tsc"]'
  __ush_match_38__len='2'
  # shellcheck disable=SC2034
  __ush_match_38__0='--tsgo-mapper'
  # shellcheck disable=SC2034
  __ush_match_38__1="${__ush_binding_57_mapper_modules}"'/@typescript/typescript-linux-x64/lib/tsc'
  __ush_count_39=${__ush_binding_39_args__len}
  __ush_index_40=0
  while [ "${__ush_index_40}" -lt "${__ush_match_38__len}" ]; do
  eval "__ush_binding_39_args__${__ush_count_39}=\"\${__ush_match_38__${__ush_index_40}}\""
    __ush_index_40=$((__ush_index_40 + 1))
    __ush_count_39=$((__ush_count_39 + 1))
  done
  __ush_binding_39_args__len=${__ush_count_39}
  __ush_at 59 'args = args + ["--tsgo-mapper-version", tsgo_mapper, "--mapper-modules", mapper_modules]'
  __ush_match_41__len='4'
  # shellcheck disable=SC2034
  __ush_match_41__0='--tsgo-mapper-version'
  # shellcheck disable=SC2034
  __ush_match_41__1="${__ush_binding_23_tsgo_mapper}"
  # shellcheck disable=SC2034
  __ush_match_41__2='--mapper-modules'
  # shellcheck disable=SC2034
  __ush_match_41__3="${__ush_binding_57_mapper_modules}"
  __ush_count_42=${__ush_binding_39_args__len}
  __ush_index_43=0
  while [ "${__ush_index_43}" -lt "${__ush_match_41__len}" ]; do
  eval "__ush_binding_39_args__${__ush_count_42}=\"\${__ush_match_41__${__ush_index_43}}\""
    __ush_index_43=$((__ush_index_43 + 1))
    __ush_count_42=$((__ush_count_42 + 1))
  done
  __ush_binding_39_args__len=${__ush_count_42}
  __ush_at 60 'args = args + version_metas $"$temp/mapper/versions.txt"'
  __ush_packed_45="$(__ush_return_path='' __ush_capture_return=1 ush_fn_version_metas "${__ush_binding_18_temp}"'/mapper/versions.txt' 4>&1 >&3)" || exit "$?"
  __ush_match_44__len=0
  ush_fn_std__unpack __ush_match_44 "${__ush_packed_45}"
  __ush_count_46=${__ush_binding_39_args__len}
  __ush_index_47=0
  while [ "${__ush_index_47}" -lt "${__ush_match_44__len}" ]; do
  eval "__ush_binding_39_args__${__ush_count_46}=\"\${__ush_match_44__${__ush_index_47}}\""
    __ush_index_47=$((__ush_index_47 + 1))
    __ush_count_46=$((__ush_count_46 + 1))
  done
  __ush_binding_39_args__len=${__ush_count_46}
fi
__ush_at 62 'for assignment in words (get VIZE_ENV) {'
__ush_value_49="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__env__get 'VIZE_ENV')" || exit "$?"
__ush_packed_50="$(__ush_return_path='' __ush_capture_return=1 ush_fn_std__string__words "${__ush_value_49}")" || exit "$?"
__ush_match_48__len=0
ush_fn_std__unpack __ush_match_48 "${__ush_packed_50}"
__ush_for_list_index_51=0
while [ "${__ush_for_list_index_51}" -lt "${__ush_match_48__len}" ]; do
  __ush_for_value_52=''
  eval "__ush_for_value_52=\"\${__ush_match_48__${__ush_for_list_index_51}}\""
  assignment="${__ush_for_value_52}"
  __ush_for_list_index_51=$((__ush_for_list_index_51 + 1))
  __ush_at 63 'args = args + ["--vize-env", $"head:$assignment", "--meta", $"env=$assignment"]'
  __ush_match_53__len='4'
  # shellcheck disable=SC2034
  __ush_match_53__0='--vize-env'
  # shellcheck disable=SC2034
  __ush_match_53__1='head:'"${assignment}"
  # shellcheck disable=SC2034
  __ush_match_53__2='--meta'
  # shellcheck disable=SC2034
  __ush_match_53__3='env='"${assignment}"
  __ush_count_54=${__ush_binding_39_args__len}
  __ush_index_55=0
  while [ "${__ush_index_55}" -lt "${__ush_match_53__len}" ]; do
  eval "__ush_binding_39_args__${__ush_count_54}=\"\${__ush_match_53__${__ush_index_55}}\""
    __ush_index_55=$((__ush_index_55 + 1))
    __ush_count_54=$((__ush_count_54 + 1))
  done
  __ush_binding_39_args__len=${__ush_count_54}
  __ush_at 64 'args = args + ["--vize-env", $"head-tsrs-fast:$assignment"]'
  __ush_match_56__len='2'
  # shellcheck disable=SC2034
  __ush_match_56__0='--vize-env'
  # shellcheck disable=SC2034
  __ush_match_56__1='head-tsrs-fast:'"${assignment}"
  __ush_count_57=${__ush_binding_39_args__len}
  __ush_index_58=0
  while [ "${__ush_index_58}" -lt "${__ush_match_56__len}" ]; do
  eval "__ush_binding_39_args__${__ush_count_57}=\"\${__ush_match_56__${__ush_index_58}}\""
    __ush_index_58=$((__ush_index_58 + 1))
    __ush_count_57=$((__ush_count_57 + 1))
  done
  __ush_binding_39_args__len=${__ush_count_57}
  __ush_at 65 'if compare_ref != "" {'
  if {
  [ "$(if [ "${__ush_binding_22_compare_ref}" != '' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
  }; then
    __ush_at 66 'args = args + ["--vize-env", $"compare:$assignment", "--vize-env", $"compare-tsrs-fast:$assignment"]'
    __ush_match_59__len='4'
    # shellcheck disable=SC2034
    __ush_match_59__0='--vize-env'
    # shellcheck disable=SC2034
    __ush_match_59__1='compare:'"${assignment}"
    # shellcheck disable=SC2034
    __ush_match_59__2='--vize-env'
    # shellcheck disable=SC2034
    __ush_match_59__3='compare-tsrs-fast:'"${assignment}"
    __ush_count_60=${__ush_binding_39_args__len}
    __ush_index_61=0
    while [ "${__ush_index_61}" -lt "${__ush_match_59__len}" ]; do
    eval "__ush_binding_39_args__${__ush_count_60}=\"\${__ush_match_59__${__ush_index_61}}\""
      __ush_index_61=$((__ush_index_61 + 1))
      __ush_count_60=$((__ush_count_60 + 1))
    done
    __ush_binding_39_args__len=${__ush_count_60}
  fi
done
__ush_at 69 'if (get TSSERVER_PLUGIN) == "true" {'
if {
__ush_value_62="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__env__get 'TSSERVER_PLUGIN')" || exit "$?"
[ "$(if [ "${__ush_value_62}" = 'true' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 70 'args = args + ["--tsserver-plugin", $"head=$temp/tsserver-plugin"]'
  __ush_match_63__len='2'
  # shellcheck disable=SC2034
  __ush_match_63__0='--tsserver-plugin'
  # shellcheck disable=SC2034
  __ush_match_63__1='head='"${__ush_binding_18_temp}"'/tsserver-plugin'
  __ush_count_64=${__ush_binding_39_args__len}
  __ush_index_65=0
  while [ "${__ush_index_65}" -lt "${__ush_match_63__len}" ]; do
  eval "__ush_binding_39_args__${__ush_count_64}=\"\${__ush_match_63__${__ush_index_65}}\""
    __ush_index_65=$((__ush_index_65 + 1))
    __ush_count_64=$((__ush_count_64 + 1))
  done
  __ush_binding_39_args__len=${__ush_count_64}
  __ush_at 71 'if compare_ref != "" {'
  if {
  [ "$(if [ "${__ush_binding_22_compare_ref}" != '' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
  }; then
    __ush_at 72 'args = args + ["--tsserver-plugin", $"compare=$temp/tsserver-plugin-compare"]'
    __ush_match_66__len='2'
    # shellcheck disable=SC2034
    __ush_match_66__0='--tsserver-plugin'
    # shellcheck disable=SC2034
    __ush_match_66__1='compare='"${__ush_binding_18_temp}"'/tsserver-plugin-compare'
    __ush_count_67=${__ush_binding_39_args__len}
    __ush_index_68=0
    while [ "${__ush_index_68}" -lt "${__ush_match_66__len}" ]; do
    eval "__ush_binding_39_args__${__ush_count_67}=\"\${__ush_match_66__${__ush_index_68}}\""
      __ush_index_68=$((__ush_index_68 + 1))
      __ush_count_67=$((__ush_count_67 + 1))
    done
    __ush_binding_39_args__len=${__ush_count_67}
  fi
fi
__ush_at 75 'args = args + version_metas $"$temp/tools/versions.txt"'
__ush_packed_70="$(__ush_return_path='' __ush_capture_return=1 ush_fn_version_metas "${__ush_binding_18_temp}"'/tools/versions.txt' 4>&1 >&3)" || exit "$?"
__ush_match_69__len=0
ush_fn_std__unpack __ush_match_69 "${__ush_packed_70}"
__ush_count_71=${__ush_binding_39_args__len}
__ush_index_72=0
while [ "${__ush_index_72}" -lt "${__ush_match_69__len}" ]; do
eval "__ush_binding_39_args__${__ush_count_71}=\"\${__ush_match_69__${__ush_index_72}}\""
  __ush_index_72=$((__ush_index_72 + 1))
  __ush_count_71=$((__ush_count_71 + 1))
done
__ush_binding_39_args__len=${__ush_count_71}
__ush_at 77 'args = args + ["--memory-reserve-gib", "16", "--meta", "memory-reserve=16GiB"]'
__ush_match_73__len='4'
# shellcheck disable=SC2034
__ush_match_73__0='--memory-reserve-gib'
# shellcheck disable=SC2034
__ush_match_73__1='16'
# shellcheck disable=SC2034
__ush_match_73__2='--meta'
# shellcheck disable=SC2034
__ush_match_73__3='memory-reserve=16GiB'
__ush_count_74=${__ush_binding_39_args__len}
__ush_index_75=0
while [ "${__ush_index_75}" -lt "${__ush_match_73__len}" ]; do
eval "__ush_binding_39_args__${__ush_count_74}=\"\${__ush_match_73__${__ush_index_75}}\""
  __ush_index_75=$((__ush_index_75 + 1))
  __ush_count_74=$((__ush_count_74 + 1))
done
__ush_binding_39_args__len=${__ush_count_74}
__ush_at 79 'free -g'
command free '-g'
__ush_at 80 'df -h /tmp $temp'
command df '-h' '/tmp' "${__ush_binding_18_temp}"
__ush_at 81 'let cpus = $env::CPUS'
__ush_env_76=${CPUS-}
if [ -z "${__ush_env_76}" ]; then __ush_report 1 'environment variable CPUS is not set or is empty'; exit 1; fi
__ush_binding_81_cpus="${__ush_env_76}"
# shellcheck disable=SC2034
cpus="${__ush_binding_81_cpus}"
__ush_at 83 'let last_cpu = to_int $cpus + -1'
__ush_value_77="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__string__to_int "${__ush_binding_81_cpus}")" || exit "$?"
__ush_binding_83_last_cpu=$((__ush_value_77 + -1))
# shellcheck disable=SC2034
last_cpu="${__ush_binding_83_last_cpu}"
__ush_at 84 'let head = git rev-parse HEAD'
__ush_value_78="$(__ush_return_path='' __ush_capture_return='1' command git 'rev-parse' 'HEAD')" || exit "$?"
__ush_binding_84_head="${__ush_value_78}"
# shellcheck disable=SC2034
head="${__ush_binding_84_head}"
__ush_at 85 'taskset -c $"0-$last_cpu" node tools/benchmarks/scripts/lsp-vs-volar.ts $args \'
__ush_match_79__len="${__ush_binding_39_args__len}"
__ush_copy_index_80=0
while [ "$__ush_copy_index_80" -lt "${__ush_binding_39_args__len}" ]; do
  eval "__ush_copy_value_81=\"\${__ush_binding_39_args__$__ush_copy_index_80}\""
  eval "__ush_match_79__${__ush_copy_index_80}=\$__ush_copy_value_81"
  __ush_copy_index_80=$((__ush_copy_index_80 + 1))
done
ush_fn_std__run_words 'taskset' s '-c' s '0-'"${__ush_binding_83_last_cpu}" s 'node' s 'tools/benchmarks/scripts/lsp-vs-volar.ts' l "$(ush_fn_std__pack __ush_match_79)" s '--tsgo' s "${__ush_binding_19_tools}"'/@typescript/typescript-linux-x64/lib/tsc' s '--volar-root' s "${__ush_binding_18_temp}"'/tools' s '--fixtures-dir' s "${__ush_binding_18_temp}"'/projects' s '--rounds' s "${__ush_binding_25_rounds}" s '--cycles' s "${__ush_binding_26_cycles}" s '--meta' s 'head='"${__ush_binding_84_head}" s '--meta' s 'cpus='"${__ush_binding_81_cpus}" s '--meta' s 'rounds='"${__ush_binding_25_rounds}" s '--meta' s 'cycles='"${__ush_binding_26_cycles}" s '--out' s 'lsp-vs-volar-'"${__ush_binding_20_job}"'.md' s '--json' s 'lsp-vs-volar-'"${__ush_binding_20_job}"'.json' s '--logs' s 'lsp-vs-volar-logs-'"${__ush_binding_20_job}"
__ush_at 98 'github::summary (read_text $"lsp-vs-volar-$job.md")'
__ush_value_83="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__fs__read_text 'lsp-vs-volar-'"${__ush_binding_20_job}"'.md')" || exit "$?"
( __ush_capture_return='0' __ush_return_path=''; ush_fn_github__summary "${__ush_value_83}" )
__ush_at 99 'if fixture_ids.contains("scale-5k") {'
if {
if __ush_std_list_contains __ush_binding_21_fixture_ids 'scale-5k'; then __ush_value_84=true; else __ush_value_84=false; fi
[ "${__ush_value_84}" = 'true' ]
}; then
  __ush_at 100 'node tools/benchmarks/scripts/lsp-vs-volar-convergence.ts \'
  command node 'tools/benchmarks/scripts/lsp-vs-volar-convergence.ts' '--json' 'lsp-vs-volar-'"${__ush_binding_20_job}"'.json' '--fixture' 'scale-5k'
fi
