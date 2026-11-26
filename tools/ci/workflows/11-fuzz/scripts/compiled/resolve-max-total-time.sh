#!/bin/sh
# Generated from '../resolve-max-total-time.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../resolve-max-total-time.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../resolve-max-total-time.ush' --checked-in -o 'resolve-max-total-time.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'resolve-max-total-time.ush'
__ush_origin='../resolve-max-total-time.ush'

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
  -h|--help) printf '%s' 'resolve-max-total-time.ush

Usage:
  resolve-max-total-time.ush [--help]
  resolve-max-total-time.ush [--man [ITEM]]
  resolve-max-total-time.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
resolve-max-total-time.ush

SYNOPSIS
resolve-max-total-time.ush [--help]
resolve-max-total-time.ush [--man [ITEM]]
resolve-max-total-time.ush [--complete [PREFIX]]
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

ush_fn_std__process__exit() {
  case $1 in ''|*[!0-9]*) return 1 ;; esac
  [ "$1" -le 255 ] || return 1
  exit "$1"
}

ush_fn_std__string__contains() {
  case $1 in *"$2"*) printf true ;; *) printf false ;; esac
}

ush_fn_std__string__to_int() {
  if __ush_integer=$(ush_fn_std____extern__int "$1" 2>/dev/null); then
    printf '%s' "$__ush_integer"
  else
    printf '%s\n' 'std::string::to_int: expected a signed 64-bit decimal integer' >&2
    return 1
  fi
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
  __ush_binding_40_end='VIZE_GITHUB_VALUE_END'
  # shellcheck disable=SC2034
  end="${__ush_binding_40_end}"
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
    __ush_value_2="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__string__contains "${__ush_fn_github__record_arg_2}" "${__ush_binding_40_end}")" || return "$?"
    [ "${__ush_value_2}" = 'true' ]
    }; then
      __ush_at 12 'fail 1 $"The value of $name contains the line that would end it"' '../../../../lib/github.ush'
      __ush_status=1
      if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
      __ush_report "$__ush_status" 'The value of '"${__ush_fn_github__record_arg_1}"' contains the line that would end it'
      return "$__ush_status"
    else
      __ush_at 14 'append_text $file $"$name<<$end\n$value\n$end\n"' '../../../../lib/github.ush'
      __ush_value_3="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__fs__append_text "${__ush_fn_github__record_arg_0}" "${__ush_fn_github__record_arg_1}"'<<'"${__ush_binding_40_end}""${__ush_lf}""${__ush_fn_github__record_arg_2}""${__ush_lf}""${__ush_binding_40_end}""${__ush_lf}")" || return "$?"
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

__ush_at 8 'if (get FUZZ_MODE) == "replay" {'
if {
__ush_value_6="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__env__get 'FUZZ_MODE')" || exit "$?"
[ "$(if [ "${__ush_value_6}" = 'replay' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 11 'github::output seconds "0"'
  ( __ush_capture_return='0' __ush_return_path=''; ush_fn_github__output 'seconds' '0' )
  __ush_at 12 'exit 0'
  ush_fn_std__process__exit 0
fi
__ush_at 15 'let event = $env::GITHUB_EVENT_NAME'
__ush_env_7=${GITHUB_EVENT_NAME-}
if [ -z "${__ush_env_7}" ]; then __ush_report 1 'environment variable GITHUB_EVENT_NAME is not set or is empty'; exit 1; fi
__ush_binding_15_event="${__ush_env_7}"
# shellcheck disable=SC2034
event="${__ush_binding_15_event}"
__ush_at 16 'let mut seconds = ""'
__ush_binding_16_seconds=''
# shellcheck disable=SC2034
seconds="${__ush_binding_16_seconds}"
__ush_at 17 'if event == "schedule" {'
if {
[ "$(if [ "${__ush_binding_15_event}" = 'schedule' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 20 'seconds = "1800"'
  __ush_binding_16_seconds='1800'
  # shellcheck disable=SC2034
  seconds="${__ush_binding_16_seconds}"
else
  __ush_at 21 '} else if event == "workflow_dispatch" {'
  if {
  [ "$(if [ "${__ush_binding_15_event}" = 'workflow_dispatch' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
  }; then
    __ush_at 22 'seconds = get REQUESTED_MAX_TOTAL_TIME'
    __ush_value_8="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__env__get 'REQUESTED_MAX_TOTAL_TIME')" || exit "$?"
    __ush_binding_16_seconds="${__ush_value_8}"
    # shellcheck disable=SC2034
    seconds="${__ush_binding_16_seconds}"
  else
    __ush_at 24 'echo "::error title=Unsupported fuzz event::campaign requires schedule or workflow_dispatch"'
    printf '%s\n' '::error title=Unsupported fuzz event::campaign requires schedule or workflow_dispatch'
    __ush_at 25 'exit 1'
    ush_fn_std__process__exit 1
  fi
fi
__ush_at 28 'if !(is_match $seconds "^[0-9]{1,4}$") || (to_int $seconds) < 1 || (to_int $seconds) > 3600 {'
if {
! {
__ush_value_9="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__regex__is_match "${__ush_binding_16_seconds}" '^[0-9]{1,4}$')" || exit "$?"
[ "${__ush_value_9}" = 'true' ]
} || {
__ush_value_10="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__string__to_int "${__ush_binding_16_seconds}")" || exit "$?"
[ "$(if [ "${__ush_value_10}" -lt 1 ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
} || {
__ush_value_11="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__string__to_int "${__ush_binding_16_seconds}")" || exit "$?"
[ "$(if [ "${__ush_value_11}" -gt 3600 ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}
}; then
  __ush_at 29 'echo "::error title=Invalid fuzz budget::max-total-time must be an integer from 1 to 3600 seconds"'
  printf '%s\n' '::error title=Invalid fuzz budget::max-total-time must be an integer from 1 to 3600 seconds'
  __ush_at 30 'exit 1'
  ush_fn_std__process__exit 1
fi
__ush_at 32 'github::output seconds $seconds'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_github__output 'seconds' "${__ush_binding_16_seconds}" )
