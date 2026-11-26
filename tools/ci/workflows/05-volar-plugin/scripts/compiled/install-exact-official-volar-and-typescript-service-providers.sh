#!/bin/sh
# Generated from '../install-exact-official-volar-and-typescript-service-providers.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../install-exact-official-volar-and-typescript-service-providers.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../install-exact-official-volar-and-typescript-service-providers.ush' --checked-in -o 'install-exact-official-volar-and-typescript-service-providers.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'install-exact-official-volar-and-typescript-service-providers.ush'
__ush_origin='../install-exact-official-volar-and-typescript-service-providers.ush'

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
  -h|--help) printf '%s' 'install-exact-official-volar-and-typescript-service-providers.ush

Usage:
  install-exact-official-volar-and-typescript-service-providers.ush [--help]
  install-exact-official-volar-and-typescript-service-providers.ush [--man [ITEM]]
  install-exact-official-volar-and-typescript-service-providers.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
install-exact-official-volar-and-typescript-service-providers.ush

SYNOPSIS
install-exact-official-volar-and-typescript-service-providers.ush [--help]
install-exact-official-volar-and-typescript-service-providers.ush [--man [ITEM]]
install-exact-official-volar-and-typescript-service-providers.ush [--complete [PREFIX]]
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

ush_fn_std__fs__mkdir_p() {
  __ush_path=$(ush_fn_std__path__resolve "$1")
  mkdir -p "$__ush_path"
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

__ush_std_fs_path() {
  __ush_path=$(ush_fn_std__path__resolve "$1" && printf .) || return "$?"
  __ush_path=${__ush_path%.}
  [ -n "$__ush_path" ] || { printf '%s\n' 'std::fs: path must not be empty' >&2; return 1; }
  case $__ush_path in .|..|/*|./*|../*) ;; *) __ush_path=./$__ush_path ;; esac
}

ush_fn_std__fs__symlink() {
  __ush_std_fs_path "$2" || return "$?"
  __ush_link=$__ush_path
  __ush_target=$(ush_fn_std__path__resolve "$1" && printf .) || return "$?"
  __ush_target=${__ush_target%.}
  [ -n "$__ush_target" ] || { printf '%s\n' 'std::fs::symlink: target must not be empty' >&2; return 1; }
  case $__ush_target in -*) __ush_target=./$__ush_target ;; esac
  ln -s "$__ush_target" "$__ush_link"
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
  __ush_binding_28_end='VIZE_GITHUB_VALUE_END'
  # shellcheck disable=SC2034
  end="${__ush_binding_28_end}"
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
    __ush_value_2="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__string__contains "${__ush_fn_github__record_arg_2}" "${__ush_binding_28_end}")" || return "$?"
    [ "${__ush_value_2}" = 'true' ]
    }; then
      __ush_at 12 'fail 1 $"The value of $name contains the line that would end it"' '../../../../lib/github.ush'
      __ush_status=1
      if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
      __ush_report "$__ush_status" 'The value of '"${__ush_fn_github__record_arg_1}"' contains the line that would end it'
      return "$__ush_status"
    else
      __ush_at 14 'append_text $file $"$name<<$end\n$value\n$end\n"' '../../../../lib/github.ush'
      __ush_value_3="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__fs__append_text "${__ush_fn_github__record_arg_0}" "${__ush_fn_github__record_arg_1}"'<<'"${__ush_binding_28_end}""${__ush_lf}""${__ush_fn_github__record_arg_2}""${__ush_lf}""${__ush_binding_28_end}""${__ush_lf}")" || return "$?"
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

__ush_at 6 'let dependencies = $"$env::RUNNER_TEMP/volar-plugin-deps"'
__ush_env_6=${RUNNER_TEMP-}
if [ -z "${__ush_env_6}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_6_dependencies="${__ush_env_6}"'/volar-plugin-deps'
# shellcheck disable=SC2034
dependencies="${__ush_binding_6_dependencies}"
__ush_at 8 'github::export_env VIZE_VOLAR_EVIDENCE_DIR $"$env::RUNNER_TEMP/volar-plugin-evidence"'
__ush_env_7=${RUNNER_TEMP-}
if [ -z "${__ush_env_7}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
( __ush_capture_return='0' __ush_return_path=''; ush_fn_github__export_env 'VIZE_VOLAR_EVIDENCE_DIR' "${__ush_env_7}"'/volar-plugin-evidence' )
__ush_at 9 'github::export_env VIZE_VOLAR_DEPENDENCY_LOCK $"$dependencies/package-lock.json"'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_github__export_env 'VIZE_VOLAR_DEPENDENCY_LOCK' "${__ush_binding_6_dependencies}"'/package-lock.json' )
__ush_at 10 'npm install --prefix $dependencies \'
command npm 'install' '--prefix' "${__ush_binding_6_dependencies}" '--ignore-scripts' '--no-audit' '--no-fund' '@volar/language-core@2.4.30' '@volar/typescript@2.4.30' 'typescript@6.0.3' 'vue@3.5.42'
__ush_at 18 'mkdir_p $"$dependencies/node_modules/@vizejs"'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__mkdir_p "${__ush_binding_6_dependencies}"'/node_modules/@vizejs' )
__ush_at 19 'symlink $"$env::GITHUB_WORKSPACE/npm/native" $"$dependencies/node_modules/@vizejs/native"'
__ush_env_8=${GITHUB_WORKSPACE-}
if [ -z "${__ush_env_8}" ]; then __ush_report 1 'environment variable GITHUB_WORKSPACE is not set or is empty'; exit 1; fi
( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__symlink "${__ush_env_8}"'/npm/native' "${__ush_binding_6_dependencies}"'/node_modules/@vizejs/native' )
__ush_at 20 'symlink $"$dependencies/node_modules" $"$env::GITHUB_WORKSPACE/npm/volar/node_modules"'
__ush_env_9=${GITHUB_WORKSPACE-}
if [ -z "${__ush_env_9}" ]; then __ush_report 1 'environment variable GITHUB_WORKSPACE is not set or is empty'; exit 1; fi
( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__symlink "${__ush_binding_6_dependencies}"'/node_modules' "${__ush_env_9}"'/npm/volar/node_modules' )
