#!/bin/sh
# Generated from '../build-the-compare-ref.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../build-the-compare-ref.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../build-the-compare-ref.ush' --checked-in -o 'build-the-compare-ref.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'build-the-compare-ref.ush'
__ush_origin='../build-the-compare-ref.ush'

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
  -h|--help) printf '%s' 'build-the-compare-ref.ush

Usage:
  build-the-compare-ref.ush [--help]
  build-the-compare-ref.ush [--man [ITEM]]
  build-the-compare-ref.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
build-the-compare-ref.ush

SYNOPSIS
build-the-compare-ref.ush [--help]
build-the-compare-ref.ush [--man [ITEM]]
build-the-compare-ref.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

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

__ush_at 4 'let compare = $"$env::RUNNER_TEMP/compare"'
__ush_env_0=${RUNNER_TEMP-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_4_compare="${__ush_env_0}"'/compare'
# shellcheck disable=SC2034
compare="${__ush_binding_4_compare}"
__ush_at 6 'let target = $"$env::GITHUB_WORKSPACE/target"'
__ush_env_1=${GITHUB_WORKSPACE-}
if [ -z "${__ush_env_1}" ]; then __ush_report 1 'environment variable GITHUB_WORKSPACE is not set or is empty'; exit 1; fi
__ush_binding_6_target="${__ush_env_1}"'/target'
# shellcheck disable=SC2034
target="${__ush_binding_6_target}"
__ush_at 8 'git fetch --no-tags --depth 1 $"https://github.com/$env::GITHUB_REPOSITORY.git" $env::COMPARE_REF'
__ush_env_2=${GITHUB_REPOSITORY-}
if [ -z "${__ush_env_2}" ]; then __ush_report 1 'environment variable GITHUB_REPOSITORY is not set or is empty'; exit 1; fi
__ush_env_3=${COMPARE_REF-}
if [ -z "${__ush_env_3}" ]; then __ush_report 1 'environment variable COMPARE_REF is not set or is empty'; exit 1; fi
command git 'fetch' '--no-tags' '--depth' 1 'https://github.com/'"${__ush_env_2}"'.git' "${__ush_env_3}"
__ush_at 9 'git worktree add $compare FETCH_HEAD'
command git 'worktree' 'add' "${__ush_binding_4_compare}" 'FETCH_HEAD'
__ush_at 10 'cd $compare {'
__ush_cwd_4=$PWD
ush_fn_std__process__chdir "${__ush_binding_4_compare}" || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
  __ush_at 11 'export CARGO_TARGET_DIR = $target {'
  __ush_had_CARGO_TARGET_DIR_5=${CARGO_TARGET_DIR+x} __ush_saved_CARGO_TARGET_DIR_5=${CARGO_TARGET_DIR-}
  export CARGO_TARGET_DIR="${__ush_binding_6_target}"
    __ush_at 12 'cargo build --profile ci-opt -p vize'
    command cargo 'build' '--profile' 'ci-opt' '-p' 'vize'
  if [ -n "$__ush_had_CARGO_TARGET_DIR_5" ]; then export CARGO_TARGET_DIR="$__ush_saved_CARGO_TARGET_DIR_5"; else unset CARGO_TARGET_DIR; fi
cd "$__ush_cwd_4" || { __ush_status=$?; __ush_report "$__ush_status" 'cannot return to the directory the cd block started in'; exit "$__ush_status"; }
__ush_at 15 'cp target/ci-opt/vize $"$env::RUNNER_TEMP/vize-compare"'
__ush_env_6=${RUNNER_TEMP-}
if [ -z "${__ush_env_6}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
command cp 'target/ci-opt/vize' "${__ush_env_6}"'/vize-compare'
__ush_at 16 'cd $compare {'
__ush_cwd_7=$PWD
ush_fn_std__process__chdir "${__ush_binding_4_compare}" || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
  __ush_at 17 'export CARGO_TARGET_DIR = $target {'
  __ush_had_CARGO_TARGET_DIR_8=${CARGO_TARGET_DIR+x} __ush_saved_CARGO_TARGET_DIR_8=${CARGO_TARGET_DIR-}
  export CARGO_TARGET_DIR="${__ush_binding_6_target}"
    __ush_at 18 'cargo build --profile ci-opt -p vize_vitrine --features napi,legacy'
    command cargo 'build' '--profile' 'ci-opt' '-p' 'vize_vitrine' '--features' 'napi,legacy'
  if [ -n "$__ush_had_CARGO_TARGET_DIR_8" ]; then export CARGO_TARGET_DIR="$__ush_saved_CARGO_TARGET_DIR_8"; else unset CARGO_TARGET_DIR; fi
cd "$__ush_cwd_7" || { __ush_status=$?; __ush_report "$__ush_status" 'cannot return to the directory the cd block started in'; exit "$__ush_status"; }
__ush_at 21 'cp target/ci-opt/libvize_vitrine.so $"$env::RUNNER_TEMP/vize-vitrine-compare.node"'
__ush_env_9=${RUNNER_TEMP-}
if [ -z "${__ush_env_9}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
command cp 'target/ci-opt/libvize_vitrine.so' "${__ush_env_9}"'/vize-vitrine-compare.node'
