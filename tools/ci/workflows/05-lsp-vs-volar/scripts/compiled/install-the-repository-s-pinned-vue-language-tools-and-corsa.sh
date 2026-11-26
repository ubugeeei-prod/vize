#!/bin/sh
# Generated from '../install-the-repository-s-pinned-vue-language-tools-and-corsa.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../install-the-repository-s-pinned-vue-language-tools-and-corsa.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../install-the-repository-s-pinned-vue-language-tools-and-corsa.ush' --checked-in -o 'install-the-repository-s-pinned-vue-language-tools-and-corsa.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'install-the-repository-s-pinned-vue-language-tools-and-corsa.ush'
__ush_origin='../install-the-repository-s-pinned-vue-language-tools-and-corsa.ush'

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
  -h|--help) printf '%s' 'install-the-repository-s-pinned-vue-language-tools-and-corsa.ush

Usage:
  install-the-repository-s-pinned-vue-language-tools-and-corsa.ush [--help]
  install-the-repository-s-pinned-vue-language-tools-and-corsa.ush [--man [ITEM]]
  install-the-repository-s-pinned-vue-language-tools-and-corsa.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
install-the-repository-s-pinned-vue-language-tools-and-corsa.ush

SYNOPSIS
install-the-repository-s-pinned-vue-language-tools-and-corsa.ush [--help]
install-the-repository-s-pinned-vue-language-tools-and-corsa.ush [--man [ITEM]]
install-the-repository-s-pinned-vue-language-tools-and-corsa.ush [--complete [PREFIX]]
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

ush_fn_std__fs__write_text() {
  __ush_path=$(ush_fn_std__path__resolve "$1")
  printf '%s' "$2" > "$__ush_path"
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

ush_fn_std__fs__mkdir() {
  __ush_std_fs_path "$1" || return "$?"
  mkdir "$__ush_path"
}

ush_fn_std__process__chdir() {
  __ush_std_fs_path "$1" || return "$?"
  CDPATH='' cd "$__ush_path" || return "$?"
}

# raises: unknown
ush_fn_catalog_pin() {
  __ush_fn_catalog_pin_arg_0="${1}"
  # shellcheck disable=SC2034
  package_name="${__ush_fn_catalog_pin_arg_0}"
  __ush_at 9 'let version = awk -v $"package_name=$package_name" -f (from_source "catalog-version.awk") pnpm-workspace.yaml'
  __ush_value_0="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__from_source 'catalog-version.awk')" || return "$?"
  __ush_value_1="$(__ush_return_path='' __ush_capture_return='1' command awk '-v' 'package_name='"${__ush_fn_catalog_pin_arg_0}" '-f' "${__ush_value_0}" 'pnpm-workspace.yaml')" || return "$?"
  __ush_binding_9_version="${__ush_value_1}"
  # shellcheck disable=SC2034
  version="${__ush_binding_9_version}"
  __ush_at 10 'if version == "" {'
  if {
  [ "$(if [ "${__ush_binding_9_version}" = '' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
  }; then
    __ush_at 11 'fail 1 $"The workspace catalog pins no version of $package_name"'
    __ush_status=1
    if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
    __ush_report "$__ush_status" 'The workspace catalog pins no version of '"${__ush_fn_catalog_pin_arg_0}"
    return "$__ush_status"
  fi
  __ush_at 13 'version'
  if [ -n "${__ush_return_path:-}" ]; then
    printf '%s' "${__ush_binding_9_version}" > "$__ush_return_path"
  elif [ "${__ush_capture_return:-0}" = '1' ]; then
    printf '%s' "${__ush_binding_9_version}" >&4
  fi
  return 0
}

__ush_at 16 'let vue = catalog_pin vue-tsc'
__ush_value_2="$(__ush_return_path='' __ush_capture_return='1' ush_fn_catalog_pin 'vue-tsc' 4>&1 >&3)" || exit "$?"
__ush_binding_16_vue="${__ush_value_2}"
# shellcheck disable=SC2034
vue="${__ush_binding_16_vue}"
__ush_at 17 'let typescript = catalog_pin typescript'
__ush_value_3="$(__ush_return_path='' __ush_capture_return='1' ush_fn_catalog_pin 'typescript' 4>&1 >&3)" || exit "$?"
__ush_binding_17_typescript="${__ush_value_3}"
# shellcheck disable=SC2034
typescript="${__ush_binding_17_typescript}"
__ush_at 18 'let corsa = catalog_pin @typescript/typescript-linux-x64'
__ush_value_4="$(__ush_return_path='' __ush_capture_return='1' ush_fn_catalog_pin '@typescript/typescript-linux-x64' 4>&1 >&3)" || exit "$?"
__ush_binding_18_corsa="${__ush_value_4}"
# shellcheck disable=SC2034
corsa="${__ush_binding_18_corsa}"
__ush_at 20 'let tsrs = node (from_source "install-the-repository-s-pinned-vue-language-tools-and-corsa.cjs")'
__ush_value_5="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__from_source 'install-the-repository-s-pinned-vue-language-tools-and-corsa.cjs')" || exit "$?"
__ush_value_6="$(__ush_return_path='' __ush_capture_return='1' command node "${__ush_value_5}")" || exit "$?"
__ush_binding_20_tsrs="${__ush_value_6}"
# shellcheck disable=SC2034
tsrs="${__ush_binding_20_tsrs}"
__ush_at 21 'if tsrs == "" {'
if {
[ "$(if [ "${__ush_binding_20_tsrs}" = '' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 22 'fail 1 "The typecheck-host manifest pins no version of tsrs"'
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'The typecheck-host manifest pins no version of tsrs'
  exit "$__ush_status"
fi
__ush_at 24 'let tools = $"$env::RUNNER_TEMP/tools"'
__ush_env_7=${RUNNER_TEMP-}
if [ -z "${__ush_env_7}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_24_tools="${__ush_env_7}"'/tools'
# shellcheck disable=SC2034
tools="${__ush_binding_24_tools}"
__ush_at 26 'mkdir $tools'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__mkdir "${__ush_binding_24_tools}" )
__ush_at 27 'cd $tools {'
__ush_cwd_8=$PWD
ush_fn_std__process__chdir "${__ush_binding_24_tools}" || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
  __ush_at 28 'write_text package.json "{\"private\":true}\n"'
  ( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__write_text 'package.json' '{"private":true}'"${__ush_lf}"'' )
  __ush_at 29 'npm install --no-audit --no-fund \'
  command npm 'install' '--no-audit' '--no-fund' '@vue/language-server@'"${__ush_binding_16_vue}" '@vue/typescript-plugin@'"${__ush_binding_16_vue}" 'typescript@'"${__ush_binding_17_typescript}" '@typescript/typescript-linux-x64@'"${__ush_binding_18_corsa}" 'tsrs@'"${__ush_binding_20_tsrs}"
  __ush_at 35 'node (from_source "install-the-repository-s-pinned-vue-language-tools-and-corsa-2.cjs") | tee versions.txt'
  __ush_value_9="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__from_source 'install-the-repository-s-pinned-vue-language-tools-and-corsa-2.cjs')" || exit "$?"
  { __ush_pipe=$(
    exec 7>&1 >&8 8>&-
    set +e
    { (command node "${__ush_value_9}") 7>&-; printf ' 1:%s' "$?" >&7; } |
    { (command tee 'versions.txt') 7>&-; printf ' 2:%s' "$?" >&7; }
  ); } 8>&1
  __ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: node (from_source "install-the-repository-s-pinned-vue-la...' 'pipeline stage 2 failed: tee versions.txt' || exit "$?"
cd "$__ush_cwd_8" || { __ush_status=$?; __ush_report "$__ush_status" 'cannot return to the directory the cd block started in'; exit "$__ush_status"; }
