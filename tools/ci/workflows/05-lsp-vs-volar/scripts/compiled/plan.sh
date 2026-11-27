#!/bin/sh
# Generated from '../plan.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../plan.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../plan.ush' --checked-in -o 'plan.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'plan.ush'
__ush_origin='../plan.ush'

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
  -h|--help) printf '%s' 'plan.ush

Usage:
  plan.ush [--help]
  plan.ush [--man [ITEM]]
  plan.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
plan.ush

SYNOPSIS
plan.ush [--help]
plan.ush [--man [ITEM]]
plan.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

exec 3>&1

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

ush_fn_std__string__contains() {
  case $1 in *"$2"*) printf true ;; *) printf false ;; esac
}

ush_fn_std__string__words() {
  set -f
  IFS=' 	
'
  # shellcheck disable=SC2086
  __ush_std_encode_args $1
}

# raises: unknown
ush_fn_github__record() {
  __ush_fn_github__record_arg_0="${1}"
  # shellcheck disable=SC2034
  file="${__ush_fn_github__record_arg_0}"
  __ush_fn_github__record_arg_1="${2}"
  # shellcheck disable=SC2034
  name="${__ush_fn_github__record_arg_1}"
  __ush_fn_github__record_arg_2="${3}"
  # shellcheck disable=SC2034
  value="${__ush_fn_github__record_arg_2}"
  __ush_at 8 'let end = "VIZE_GITHUB_VALUE_END"' '../../../../lib/github.ush'
  __ush_binding_17_end='VIZE_GITHUB_VALUE_END'
  # shellcheck disable=SC2034
  end="${__ush_binding_17_end}"
  __ush_at 9 'if !(contains $value "\n") {' '../../../../lib/github.ush'
  if ! {
  __ush_value_0="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__string__contains "${__ush_fn_github__record_arg_2}" ''"${__ush_lf}"'')" || return "$?"
  [ "${__ush_value_0}" = 'true' ]
  }; then
    __ush_at 10 'append_text $file $"$name=$value\n"' '../../../../lib/github.ush'
    __ush_value_1="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__fs__append_text "${__ush_fn_github__record_arg_0}" "${__ush_fn_github__record_arg_1}"'='"${__ush_fn_github__record_arg_2}""${__ush_lf}")" || return "$?"
    if [ -n "${__ush_return_path:-}" ]; then
      printf '%s' '' > "$__ush_return_path"
    elif [ "${__ush_capture_return:-0}" = '1' ]; then
      printf '%s' '' >&4
    fi
    return 0
  else
    __ush_at 11 '} else if contains $value $end {' '../../../../lib/github.ush'
    if {
    __ush_value_2="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__string__contains "${__ush_fn_github__record_arg_2}" "${__ush_binding_17_end}")" || return "$?"
    [ "${__ush_value_2}" = 'true' ]
    }; then
      __ush_at 12 'fail 1 $"The value of $name contains the line that would end it"' '../../../../lib/github.ush'
      __ush_status=1
      if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
      __ush_report "$__ush_status" 'The value of '"${__ush_fn_github__record_arg_1}"' contains the line that would end it'
      return "$__ush_status"
    else
      __ush_at 14 'append_text $file $"$name<<$end\n$value\n$end\n"' '../../../../lib/github.ush'
      __ush_value_3="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__fs__append_text "${__ush_fn_github__record_arg_0}" "${__ush_fn_github__record_arg_1}"'<<'"${__ush_binding_17_end}""${__ush_lf}""${__ush_fn_github__record_arg_2}""${__ush_lf}""${__ush_binding_17_end}""${__ush_lf}")" || return "$?"
      if [ -n "${__ush_return_path:-}" ]; then
        printf '%s' '' > "$__ush_return_path"
      elif [ "${__ush_capture_return:-0}" = '1' ]; then
        printf '%s' '' >&4
      fi
      return 0
    fi
  fi
}

# raises: unknown
ush_fn_github__output() {
  __ush_fn_github__output_arg_0="${1}"
  # shellcheck disable=SC2034
  name="${__ush_fn_github__output_arg_0}"
  __ush_fn_github__output_arg_1="${2}"
  # shellcheck disable=SC2034
  value="${__ush_fn_github__output_arg_1}"
  __ush_at 20 'record $env::GITHUB_OUTPUT $name $value' '../../../../lib/github.ush'
  __ush_env_4=${GITHUB_OUTPUT-}
  if [ -z "${__ush_env_4}" ]; then __ush_report 1 'environment variable GITHUB_OUTPUT is not set or is empty'; return 1; fi
  __ush_value_5="$(__ush_return_path='' __ush_capture_return='1' ush_fn_github__record "${__ush_env_4}" "${__ush_fn_github__output_arg_0}" "${__ush_fn_github__output_arg_1}" 4>&1 >&3)" || return "$?"
  if [ -n "${__ush_return_path:-}" ]; then
    printf '%s' '' > "$__ush_return_path"
  elif [ "${__ush_capture_return:-0}" = '1' ]; then
    printf '%s' '' >&4
  fi
  return 0
}

__ush_at 6 'let fixtures = [argument for id in words $env::FIXTURES for argument in ["--fixture", id]]'
__ush_comprehension_count_6=0
__ush_env_8=${FIXTURES-}
if [ -z "${__ush_env_8}" ]; then __ush_report 1 'environment variable FIXTURES is not set or is empty'; exit 1; fi
__ush_packed_9="$(__ush_return_path='' __ush_capture_return=1 ush_fn_std__string__words "${__ush_env_8}")" || exit "$?"
__ush_match_7__len=0
ush_fn_std__unpack __ush_match_7 "${__ush_packed_9}"
__ush_comprehension_index_10=0
while [ "${__ush_comprehension_index_10}" -lt "${__ush_match_7__len}" ]; do
eval "__ush_comprehension_item_11=\"\${__ush_match_7__${__ush_comprehension_index_10}}\""
__ush_match_12__len='2'
# shellcheck disable=SC2034
__ush_match_12__0='--fixture'
# shellcheck disable=SC2034
__ush_match_12__1="${__ush_comprehension_item_11}"
__ush_comprehension_index_13=0
while [ "${__ush_comprehension_index_13}" -lt "${__ush_match_12__len}" ]; do
eval "__ush_comprehension_item_14=\"\${__ush_match_12__${__ush_comprehension_index_13}}\""
# shellcheck disable=SC2034
__ush_comprehension_value_15="${__ush_comprehension_item_14}"
eval "__ush_binding_6_fixtures__${__ush_comprehension_count_6}=\$__ush_comprehension_value_15"
__ush_comprehension_count_6=$((__ush_comprehension_count_6 + 1))
__ush_comprehension_index_13=$((__ush_comprehension_index_13 + 1))
done
__ush_comprehension_index_10=$((__ush_comprehension_index_10 + 1))
done
__ush_binding_6_fixtures__len="${__ush_comprehension_count_6}"
__ush_at 7 'let jobs = node tools/benchmarks/scripts/lsp-vs-volar.ts --plan $fixtures'
__ush_match_16__len="${__ush_binding_6_fixtures__len}"
__ush_copy_index_17=0
while [ "$__ush_copy_index_17" -lt "${__ush_binding_6_fixtures__len}" ]; do
  eval "__ush_copy_value_18=\"\${__ush_binding_6_fixtures__$__ush_copy_index_17}\""
  eval "__ush_match_16__${__ush_copy_index_17}=\$__ush_copy_value_18"
  __ush_copy_index_17=$((__ush_copy_index_17 + 1))
done
__ush_value_20="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__run_words 'node' s 'tools/benchmarks/scripts/lsp-vs-volar.ts' s '--plan' l "$(ush_fn_std__pack __ush_match_16)")" || exit "$?"
__ush_binding_7_jobs="${__ush_value_20}"
# shellcheck disable=SC2034
jobs="${__ush_binding_7_jobs}"
__ush_at 9 'github::output jobs $jobs'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_github__output 'jobs' "${__ush_binding_7_jobs}" )
