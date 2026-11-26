#!/bin/sh
# Generated from '../install-pinned-neovim.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../install-pinned-neovim.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../install-pinned-neovim.ush' --checked-in -o 'install-pinned-neovim.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'install-pinned-neovim.ush'
__ush_origin='../install-pinned-neovim.ush'

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
  -h|--help) printf '%s' 'install-pinned-neovim.ush

Usage:
  install-pinned-neovim.ush [--help]
  install-pinned-neovim.ush [--man [ITEM]]
  install-pinned-neovim.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
install-pinned-neovim.ush

SYNOPSIS
install-pinned-neovim.ush [--help]
install-pinned-neovim.ush [--man [ITEM]]
install-pinned-neovim.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

exec 3>&1

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
  __ush_binding_23_end='VIZE_GITHUB_VALUE_END'
  # shellcheck disable=SC2034
  end="${__ush_binding_23_end}"
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
    __ush_value_2="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__string__contains "${__ush_fn_github__record_arg_2}" "${__ush_binding_23_end}")" || return "$?"
    [ "${__ush_value_2}" = 'true' ]
    }; then
      __ush_at 12 'fail 1 $"The value of $name contains the line that would end it"' '../../../../lib/github.ush'
      __ush_status=1
      if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
      __ush_report "$__ush_status" 'The value of '"${__ush_fn_github__record_arg_1}"' contains the line that would end it'
      return "$__ush_status"
    else
      __ush_at 14 'append_text $file $"$name<<$end\n$value\n$end\n"' '../../../../lib/github.ush'
      __ush_value_3="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__fs__append_text "${__ush_fn_github__record_arg_0}" "${__ush_fn_github__record_arg_1}"'<<'"${__ush_binding_23_end}""${__ush_lf}""${__ush_fn_github__record_arg_2}""${__ush_lf}""${__ush_binding_23_end}""${__ush_lf}")" || return "$?"
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

__ush_at 6 'let archive = $"$env::RUNNER_TEMP/nvim.tar.gz"'
__ush_env_6=${RUNNER_TEMP-}
if [ -z "${__ush_env_6}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_6_archive="${__ush_env_6}"'/nvim.tar.gz'
# shellcheck disable=SC2034
archive="${__ush_binding_6_archive}"
__ush_at 7 'let nvim = $"$env::RUNNER_TEMP/nvim-linux-x86_64/bin/nvim"'
__ush_env_7=${RUNNER_TEMP-}
if [ -z "${__ush_env_7}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_7_nvim="${__ush_env_7}"'/nvim-linux-x86_64/bin/nvim'
# shellcheck disable=SC2034
nvim="${__ush_binding_7_nvim}"
__ush_at 9 'bash .github/actions/vscode-host-smoke/download-artifact.sh \'
__ush_env_8=${NVIM_VERSION-}
if [ -z "${__ush_env_8}" ]; then __ush_report 1 'environment variable NVIM_VERSION is not set or is empty'; exit 1; fi
__ush_env_9=${NVIM_SHA256-}
if [ -z "${__ush_env_9}" ]; then __ush_report 1 'environment variable NVIM_SHA256 is not set or is empty'; exit 1; fi
command bash '.github/actions/vscode-host-smoke/download-artifact.sh' 'https://github.com/neovim/neovim/releases/download/'"${__ush_env_8}"'/nvim-linux-x86_64.tar.gz' "${__ush_binding_6_archive}" "${__ush_env_9}"
__ush_at 13 'tar -C $env::RUNNER_TEMP -xzf $archive'
__ush_env_10=${RUNNER_TEMP-}
if [ -z "${__ush_env_10}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
command tar '-C' "${__ush_env_10}" '-xzf' "${__ush_binding_6_archive}"
__ush_at 14 'github::export_env VIZE_TEST_NVIM_PATH $nvim'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_github__export_env 'VIZE_TEST_NVIM_PATH' "${__ush_binding_7_nvim}" )
__ush_at 15 'exec $nvim ["--version"] | head -1'
__ush_match_11__len='1'
# shellcheck disable=SC2034
__ush_match_11__0='--version'
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (ush_fn_std__process__exec "${__ush_binding_7_nvim}" "$(ush_fn_std__pack __ush_match_11)") 7>&-; printf ' 1:%s' "$?" >&7; } |
  { (command head -1) 7>&-; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: exec $nvim ["--version"]' 'pipeline stage 2 failed: head -1' || exit "$?"
