#!/bin/sh
# Generated from '../build-the-actual-reviewed-native-source.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../build-the-actual-reviewed-native-source.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../build-the-actual-reviewed-native-source.ush' --checked-in -o 'build-the-actual-reviewed-native-source.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'build-the-actual-reviewed-native-source.ush'
__ush_origin='../build-the-actual-reviewed-native-source.ush'

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
  -h|--help) printf '%s' 'build-the-actual-reviewed-native-source.ush

Usage:
  build-the-actual-reviewed-native-source.ush [--help]
  build-the-actual-reviewed-native-source.ush [--man [ITEM]]
  build-the-actual-reviewed-native-source.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
build-the-actual-reviewed-native-source.ush

SYNOPSIS
build-the-actual-reviewed-native-source.ush [--help]
build-the-actual-reviewed-native-source.ush [--man [ITEM]]
build-the-actual-reviewed-native-source.ush [--complete [PREFIX]]
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

__ush_at 5 'let addon = "npm/native/vize-vitrine.linux-x64-gnu.node"'
__ush_binding_5_addon='npm/native/vize-vitrine.linux-x64-gnu.node'
# shellcheck disable=SC2034
addon="${__ush_binding_5_addon}"
__ush_at 7 'let checkout = git rev-parse HEAD'
__ush_value_6="$(__ush_return_path='' __ush_capture_return='1' command git 'rev-parse' 'HEAD')" || exit "$?"
__ush_binding_7_checkout="${__ush_value_6}"
# shellcheck disable=SC2034
checkout="${__ush_binding_7_checkout}"
__ush_at 8 'if $checkout != $env::VIZE_SOURCE_SHA {'
if {
__ush_env_7=${VIZE_SOURCE_SHA-}
if [ -z "${__ush_env_7}" ]; then __ush_report 1 'environment variable VIZE_SOURCE_SHA is not set or is empty'; exit 1; fi
[ "$(if [ "${__ush_binding_7_checkout}" != "${__ush_env_7}" ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 9 'fail 1 $"The checkout is $checkout, not the reviewed source $env::VIZE_SOURCE_SHA"'
  __ush_env_8=${VIZE_SOURCE_SHA-}
  if [ -z "${__ush_env_8}" ]; then __ush_report 1 'environment variable VIZE_SOURCE_SHA is not set or is empty'; exit 1; fi
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'The checkout is '"${__ush_binding_7_checkout}"', not the reviewed source '"${__ush_env_8}"
  exit "$__ush_status"
fi
__ush_at 11 'let token = openssl rand -hex 16'
__ush_value_9="$(__ush_return_path='' __ush_capture_return='1' command openssl 'rand' '-hex' 16)" || exit "$?"
__ush_binding_11_token="${__ush_value_9}"
# shellcheck disable=SC2034
token="${__ush_binding_11_token}"
__ush_at 12 'export VIZE_NATIVE_BUILD_TOKEN = $token {'
__ush_had_VIZE_NATIVE_BUILD_TOKEN_10=${VIZE_NATIVE_BUILD_TOKEN+x} __ush_saved_VIZE_NATIVE_BUILD_TOKEN_10=${VIZE_NATIVE_BUILD_TOKEN-}
export VIZE_NATIVE_BUILD_TOKEN="${__ush_binding_11_token}"
  __ush_at 13 'cargo build --profile ci -p vize_vitrine --features napi'
  command cargo 'build' '--profile' 'ci' '-p' 'vize_vitrine' '--features' 'napi'
if [ -n "$__ush_had_VIZE_NATIVE_BUILD_TOKEN_10" ]; then export VIZE_NATIVE_BUILD_TOKEN="$__ush_saved_VIZE_NATIVE_BUILD_TOKEN_10"; else unset VIZE_NATIVE_BUILD_TOKEN; fi
__ush_at 15 'cp target/ci/libvize_vitrine.so $addon'
command cp 'target/ci/libvize_vitrine.so' "${__ush_binding_5_addon}"
__ush_at 16 'github::export_env VIZE_VOLAR_EXPECTED_NATIVE_TOKEN $token'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_github__export_env 'VIZE_VOLAR_EXPECTED_NATIVE_TOKEN' "${__ush_binding_11_token}" )
__ush_at 17 'github::export_env NAPI_RS_NATIVE_LIBRARY_PATH $"$env::GITHUB_WORKSPACE/$addon"'
__ush_env_11=${GITHUB_WORKSPACE-}
if [ -z "${__ush_env_11}" ]; then __ush_report 1 'environment variable GITHUB_WORKSPACE is not set or is empty'; exit 1; fi
( __ush_capture_return='0' __ush_return_path=''; ush_fn_github__export_env 'NAPI_RS_NATIVE_LIBRARY_PATH' "${__ush_env_11}"'/'"${__ush_binding_5_addon}" )
