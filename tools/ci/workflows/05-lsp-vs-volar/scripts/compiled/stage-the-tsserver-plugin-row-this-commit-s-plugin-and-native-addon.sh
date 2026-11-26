#!/bin/sh
# Generated from '../stage-the-tsserver-plugin-row-this-commit-s-plugin-and-native-addon.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../stage-the-tsserver-plugin-row-this-commit-s-plugin-and-native-addon.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../stage-the-tsserver-plugin-row-this-commit-s-plugin-and-native-addon.ush' --checked-in -o 'stage-the-tsserver-plugin-row-this-commit-s-plugin-and-native-addon.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'stage-the-tsserver-plugin-row-this-commit-s-plugin-and-native-addon.ush'
__ush_origin='../stage-the-tsserver-plugin-row-this-commit-s-plugin-and-native-addon.ush'

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
  -h|--help) printf '%s' 'stage-the-tsserver-plugin-row-this-commit-s-plugin-and-native-addon.ush

Usage:
  stage-the-tsserver-plugin-row-this-commit-s-plugin-and-native-addon.ush [--help]
  stage-the-tsserver-plugin-row-this-commit-s-plugin-and-native-addon.ush [--man [ITEM]]
  stage-the-tsserver-plugin-row-this-commit-s-plugin-and-native-addon.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
stage-the-tsserver-plugin-row-this-commit-s-plugin-and-native-addon.ush

SYNOPSIS
stage-the-tsserver-plugin-row-this-commit-s-plugin-and-native-addon.ush [--help]
stage-the-tsserver-plugin-row-this-commit-s-plugin-and-native-addon.ush [--man [ITEM]]
stage-the-tsserver-plugin-row-this-commit-s-plugin-and-native-addon.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

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

__ush_std_fs_path() {
  __ush_path=$(ush_fn_std__path__resolve "$1" && printf .) || return "$?"
  __ush_path=${__ush_path%.}
  [ -n "$__ush_path" ] || { printf '%s\n' 'std::fs: path must not be empty' >&2; return 1; }
  case $__ush_path in .|..|/*|./*|../*) ;; *) __ush_path=./$__ush_path ;; esac
}

ush_fn_std__fs__remove_tree() {
  __ush_std_fs_path "$1" || return "$?"
  rm -rf "$__ush_path"
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
ush_fn_stage_plugin() {
  __ush_fn_stage_plugin_arg_0="${1}"
  # shellcheck disable=SC2034
  root="${__ush_fn_stage_plugin_arg_0}"
  __ush_fn_stage_plugin_arg_1="${2}"
  # shellcheck disable=SC2034
  packages="${__ush_fn_stage_plugin_arg_1}"
  __ush_fn_stage_plugin_arg_2="${3}"
  # shellcheck disable=SC2034
  addon="${__ush_fn_stage_plugin_arg_2}"
  __ush_at 8 'let scope = $"$root/@vizejs"'
  __ush_binding_8_scope="${__ush_fn_stage_plugin_arg_0}"'/@vizejs'
  # shellcheck disable=SC2034
  scope="${__ush_binding_8_scope}"
  __ush_at 9 'mkdir_p $scope'
  ( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__mkdir_p "${__ush_binding_8_scope}" ) || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
  __ush_at 10 'cp -R $"$packages/native" $"$scope/native"'
  command cp '-R' "${__ush_fn_stage_plugin_arg_1}"'/native' "${__ush_binding_8_scope}"'/native' || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
  __ush_at 11 'cp $addon $"$scope/native/vize-vitrine.linux-x64-gnu.node"'
  command cp "${__ush_fn_stage_plugin_arg_2}" "${__ush_binding_8_scope}"'/native/vize-vitrine.linux-x64-gnu.node' || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
  __ush_at 12 'cp -R $"$packages/typescript-plugin" $"$scope/typescript-vue-plugin"'
  command cp '-R' "${__ush_fn_stage_plugin_arg_1}"'/typescript-plugin' "${__ush_binding_8_scope}"'/typescript-vue-plugin' || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
  __ush_at 13 'remove_tree $"$scope/native/node_modules"'
  ( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__remove_tree "${__ush_binding_8_scope}"'/native/node_modules' ) || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
  __ush_at 14 'remove_tree $"$scope/typescript-vue-plugin/node_modules"'
  ( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__remove_tree "${__ush_binding_8_scope}"'/typescript-vue-plugin/node_modules' ) || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
  __ush_at 16 'symlink $"$env::RUNNER_TEMP/tools/node_modules/typescript" $"$root/typescript"'
  __ush_env_0=${RUNNER_TEMP-}
  if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; return 1; fi
  ( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__symlink "${__ush_env_0}"'/tools/node_modules/typescript' "${__ush_fn_stage_plugin_arg_0}"'/typescript' ) || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
}

__ush_at 20 'cargo build --profile ci-opt -p vize_vitrine --features napi,legacy'
command cargo 'build' '--profile' 'ci-opt' '-p' 'vize_vitrine' '--features' 'napi,legacy'
__ush_at 21 'stage_plugin $"$env::RUNNER_TEMP/tsserver-plugin/node_modules" npm target/ci-opt/libvize_vitrine.so'
__ush_env_1=${RUNNER_TEMP-}
if [ -z "${__ush_env_1}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
( __ush_capture_return='0' __ush_return_path=''; ush_fn_stage_plugin "${__ush_env_1}"'/tsserver-plugin/node_modules' 'npm' 'target/ci-opt/libvize_vitrine.so' )
__ush_at 22 'if (get COMPARE_REF) != "" {'
if {
__ush_value_2="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__env__get 'COMPARE_REF')" || exit "$?"
[ "$(if [ "${__ush_value_2}" != '' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 23 'stage_plugin \'
  __ush_env_3=${RUNNER_TEMP-}
  if [ -z "${__ush_env_3}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
  __ush_env_4=${RUNNER_TEMP-}
  if [ -z "${__ush_env_4}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
  __ush_env_5=${RUNNER_TEMP-}
  if [ -z "${__ush_env_5}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
  ( __ush_capture_return='0' __ush_return_path=''; ush_fn_stage_plugin "${__ush_env_3}"'/tsserver-plugin-compare/node_modules' "${__ush_env_4}"'/compare/npm' "${__ush_env_5}"'/vize-vitrine-compare.node' )
fi
