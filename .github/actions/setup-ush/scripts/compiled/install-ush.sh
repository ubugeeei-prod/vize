#!/bin/sh
# Generated from '../install-ush.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../install-ush.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../install-ush.ush' --checked-in -o 'install-ush.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'install-ush.ush'
__ush_origin='../install-ush.ush'

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
  -h|--help) printf '%s' 'install-ush.ush

Usage:
  install-ush.ush [--help]
  install-ush.ush [--man [ITEM]]
  install-ush.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
install-ush.ush

SYNOPSIS
install-ush.ush [--help]
install-ush.ush [--man [ITEM]]
install-ush.ush [--complete [PREFIX]]
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

__ush_std_fs_path() {
  __ush_path=$(ush_fn_std__path__resolve "$1" && printf .) || return "$?"
  __ush_path=${__ush_path%.}
  [ -n "$__ush_path" ] || { printf '%s\n' 'std::fs: path must not be empty' >&2; return 1; }
  case $__ush_path in .|..|/*|./*|../*) ;; *) __ush_path=./$__ush_path ;; esac
}

ush_fn_std__process__chdir() {
  __ush_std_fs_path "$1" || return "$?"
  CDPATH='' cd "$__ush_path" || return "$?"
}

# raises: unknown
ush_fn_github__add_path() {
  __ush_fn_github__add_path_arg_0="${1}"
  # shellcheck disable=SC2034
  directory="${__ush_fn_github__add_path_arg_0}"
  __ush_at 30 'append_text $env::GITHUB_PATH $"$directory\n"' '../../../../../tools/ci/lib/github.ush'
  __ush_env_0=${GITHUB_PATH-}
  if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable GITHUB_PATH is not set or is empty'; return 1; fi
  __ush_value_1="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__fs__append_text "${__ush_env_0}" "${__ush_fn_github__add_path_arg_0}""${__ush_lf}")" || return "$?"
  if [ -n "${__ush_return_path:-}" ]; then
    printf '%s' '' > "$__ush_return_path"
  elif [ "${__ush_capture_return:-0}" = '1' ]; then
    printf '%s' '' >&4
  fi
  return 0
}

__ush_at 6 'let version = "0.19.0"'
__ush_binding_6_version='0.19.0'
# shellcheck disable=SC2034
version="${__ush_binding_6_version}"
__ush_at 9 'let checksums = """'
__ush_binding_9_checksums='eca9457ad433c32abfef778e22aee1744306674c1603b7590a51c666f76dba52  ush-aarch64-apple-darwin.tar.gz'"${__ush_lf}"'001a554b6e8950ac662b3438b468b826f6c1039879265c1155a242abc3e98a33  ush-aarch64-unknown-linux-gnu.tar.gz'"${__ush_lf}"'d675dc61ad0ffe5767d91113f542085c2c7d7db118230065baaa7db903c87328  ush-x86_64-apple-darwin.tar.gz'"${__ush_lf}"'e91b2f3d96f6e03bd5885317ab9498b2a85dbac8888ed9efcde791adfb2fae80  ush-x86_64-unknown-linux-gnu.tar.gz'
# shellcheck disable=SC2034
checksums="${__ush_binding_9_checksums}"
__ush_at 16 'if $env::RUNNER_OS != "Linux" {'
if {
__ush_env_2=${RUNNER_OS-}
if [ -z "${__ush_env_2}" ]; then __ush_report 1 'environment variable RUNNER_OS is not set or is empty'; exit 1; fi
[ "$(if [ "${__ush_env_2}" != 'Linux' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 17 'fail 1 $"setup-ush installs ush on Linux runners, not on $env::RUNNER_OS"'
  __ush_env_3=${RUNNER_OS-}
  if [ -z "${__ush_env_3}" ]; then __ush_report 1 'environment variable RUNNER_OS is not set or is empty'; exit 1; fi
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'setup-ush installs ush on Linux runners, not on '"${__ush_env_3}"
  exit "$__ush_status"
fi
__ush_at 19 'let target = match $env::RUNNER_ARCH {'
__ush_env_5=${RUNNER_ARCH-}
if [ -z "${__ush_env_5}" ]; then __ush_report 1 'environment variable RUNNER_ARCH is not set or is empty'; exit 1; fi
__ush_match_4="${__ush_env_5}"
case "${__ush_match_4}" in
  'X64')
    __ush_at 20 '"X64" => "x86_64-unknown-linux-gnu"'
    __ush_binding_19_target='x86_64-unknown-linux-gnu'
    ;;
  'ARM64')
    __ush_at 21 '"ARM64" => "aarch64-unknown-linux-gnu"'
    __ush_binding_19_target='aarch64-unknown-linux-gnu'
    ;;
  *)
    arch="${__ush_match_4-}"
    __ush_at 22 'arch => fail 1 $"setup-ush has no ush build for $arch runners"'
    __ush_status=1
    if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
    __ush_report "$__ush_status" 'setup-ush has no ush build for '"${arch}"' runners'
    exit "$__ush_status"
    ;;
esac
# shellcheck disable=SC2034
target="${__ush_binding_19_target}"
__ush_at 24 'let name = $"ush-$target"'
__ush_binding_24_name='ush-'"${__ush_binding_19_target}"
# shellcheck disable=SC2034
name="${__ush_binding_24_name}"
__ush_at 25 'let url = $"https://github.com/ubugeeei-prod/ush/releases/download/v$version/$name.tar.gz"'
__ush_binding_25_url='https://github.com/ubugeeei-prod/ush/releases/download/v'"${__ush_binding_6_version}"'/'"${__ush_binding_24_name}"'.tar.gz'
# shellcheck disable=SC2034
url="${__ush_binding_25_url}"
__ush_at 27 'cd $env::RUNNER_TEMP {'
__ush_env_6=${RUNNER_TEMP-}
if [ -z "${__ush_env_6}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_cwd_7=$PWD
ush_fn_std__process__chdir "${__ush_env_6}" || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
  __ush_at 28 'curl --fail --silent --show-error --location --retry 5 --retry-all-errors --output $"$name.tar.gz" $url'
  command curl '--fail' '--silent' '--show-error' '--location' '--retry' 5 '--retry-all-errors' '--output' "${__ush_binding_24_name}"'.tar.gz' "${__ush_binding_25_url}"
  __ush_at 30 '$"$checksums\n" | sha256sum --check --strict --ignore-missing'
  { __ush_pipe=$(
    exec 7>&1 >&8 8>&-
    set +e
    { (printf '%s' "${__ush_binding_9_checksums}""${__ush_lf}") 7>&-; printf ' 1:%s' "$?" >&7; } |
    { (command sha256sum '--check' '--strict' '--ignore-missing') 7>&-; printf ' 2:%s' "$?" >&7; }
  ); } 8>&1
  __ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: $"$checksums\n"' 'pipeline stage 2 failed: sha256sum --check --strict --ignore-missing' || exit "$?"
  __ush_at 31 'tar -xzf $"$name.tar.gz"'
  command tar '-xzf' "${__ush_binding_24_name}"'.tar.gz'
cd "$__ush_cwd_7" || { __ush_status=$?; __ush_report "$__ush_status" 'cannot return to the directory the cd block started in'; exit "$__ush_status"; }
__ush_at 33 'github::add_path $"$env::RUNNER_TEMP/$name"'
__ush_env_8=${RUNNER_TEMP-}
if [ -z "${__ush_env_8}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
( __ush_capture_return='0' __ush_return_path=''; ush_fn_github__add_path "${__ush_env_8}"'/'"${__ush_binding_24_name}" )
