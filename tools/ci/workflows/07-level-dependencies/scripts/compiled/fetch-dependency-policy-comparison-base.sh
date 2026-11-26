#!/bin/sh
# Generated from '../fetch-dependency-policy-comparison-base.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../fetch-dependency-policy-comparison-base.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../fetch-dependency-policy-comparison-base.ush' --checked-in -o 'fetch-dependency-policy-comparison-base.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'fetch-dependency-policy-comparison-base.ush'
__ush_origin='../fetch-dependency-policy-comparison-base.ush'

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
  -h|--help) printf '%s' 'fetch-dependency-policy-comparison-base.ush

Usage:
  fetch-dependency-policy-comparison-base.ush [--help]
  fetch-dependency-policy-comparison-base.ush [--man [ITEM]]
  fetch-dependency-policy-comparison-base.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
fetch-dependency-policy-comparison-base.ush

SYNOPSIS
fetch-dependency-policy-comparison-base.ush [--help]
fetch-dependency-policy-comparison-base.ush [--man [ITEM]]
fetch-dependency-policy-comparison-base.ush [--complete [PREFIX]]
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

ush_fn_std__regex__is_match() {
  VALUE="$1" PATTERN="$2" awk 'BEGIN { value = ENVIRON["VALUE"]; pattern = ENVIRON["PATTERN"]; if (value ~ pattern) { printf "true" } else { printf "false" } }'
}

ush_fn_std__string__contains() {
  case $1 in *"$2"*) printf true ;; *) printf false ;; esac
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
  __ush_binding_25_end='VIZE_GITHUB_VALUE_END'
  # shellcheck disable=SC2034
  end="${__ush_binding_25_end}"
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
    __ush_value_2="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__string__contains "${__ush_fn_github__record_arg_2}" "${__ush_binding_25_end}")" || return "$?"
    [ "${__ush_value_2}" = 'true' ]
    }; then
      __ush_at 12 'fail 1 $"The value of $name contains the line that would end it"' '../../../../lib/github.ush'
      __ush_status=1
      if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
      __ush_report "$__ush_status" 'The value of '"${__ush_fn_github__record_arg_1}"' contains the line that would end it'
      return "$__ush_status"
    else
      __ush_at 14 'append_text $file $"$name<<$end\n$value\n$end\n"' '../../../../lib/github.ush'
      __ush_value_3="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__fs__append_text "${__ush_fn_github__record_arg_0}" "${__ush_fn_github__record_arg_1}"'<<'"${__ush_binding_25_end}""${__ush_lf}""${__ush_fn_github__record_arg_2}""${__ush_lf}""${__ush_binding_25_end}""${__ush_lf}")" || return "$?"
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
ush_fn_github__export_env() {
  __ush_fn_github__export_env_arg_0="${1}"
  # shellcheck disable=SC2034
  name="${__ush_fn_github__export_env_arg_0}"
  __ush_fn_github__export_env_arg_1="${2}"
  # shellcheck disable=SC2034
  value="${__ush_fn_github__export_env_arg_1}"
  __ush_at 25 'record $env::GITHUB_ENV $name $value' '../../../../lib/github.ush'
  __ush_env_4=${GITHUB_ENV-}
  if [ -z "${__ush_env_4}" ]; then __ush_report 1 'environment variable GITHUB_ENV is not set or is empty'; return 1; fi
  __ush_value_5="$(__ush_return_path='' __ush_capture_return='1' ush_fn_github__record "${__ush_env_4}" "${__ush_fn_github__export_env_arg_0}" "${__ush_fn_github__export_env_arg_1}" 4>&1 >&3)" || return "$?"
  if [ -n "${__ush_return_path:-}" ]; then
    printf '%s' '' > "$__ush_return_path"
  elif [ "${__ush_capture_return:-0}" = '1' ]; then
    printf '%s' '' >&4
  fi
  return 0
}

__ush_at 7 'let requested = get BASE_SHA'
__ush_value_6="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__env__get 'BASE_SHA')" || exit "$?"
__ush_binding_7_requested="${__ush_value_6}"
# shellcheck disable=SC2034
requested="${__ush_binding_7_requested}"
__ush_at 8 'let dependency_base = match requested {'
case "${__ush_binding_7_requested}" in
  '')
    __ush_at 9 '"" => git rev-parse HEAD^'
    __ush_value_7="$(__ush_return_path='' __ush_capture_return='1' command git 'rev-parse' 'HEAD^')" || exit "$?"
    __ush_binding_8_dependency_base="${__ush_value_7}"
    ;;
  *)
    sha="${__ush_binding_7_requested-}"
    __ush_at 10 'sha => sha'
    __ush_binding_8_dependency_base="${sha}"
    ;;
esac
# shellcheck disable=SC2034
dependency_base="${__ush_binding_8_dependency_base}"
__ush_at 13 'if !(is_match $dependency_base "^[0-9a-f]{40}$") || is_match $dependency_base "^0+$" {'
if {
! {
__ush_value_8="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__regex__is_match "${__ush_binding_8_dependency_base}" '^[0-9a-f]{40}$')" || exit "$?"
[ "${__ush_value_8}" = 'true' ]
} || {
__ush_value_9="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__regex__is_match "${__ush_binding_8_dependency_base}" '^0+$')" || exit "$?"
[ "${__ush_value_9}" = 'true' ]
}
}; then
  __ush_at 14 'fail 1 "Missing dependency policy comparison base"'
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'Missing dependency policy comparison base'
  exit "$__ush_status"
fi
__ush_at 16 'git fetch --no-tags --depth=1 origin $dependency_base'
command git 'fetch' '--no-tags' '--depth=1' 'origin' "${__ush_binding_8_dependency_base}"
__ush_at 17 'github::export_env DEPENDENCY_BASE $dependency_base'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_github__export_env 'DEPENDENCY_BASE' "${__ush_binding_8_dependency_base}" )
