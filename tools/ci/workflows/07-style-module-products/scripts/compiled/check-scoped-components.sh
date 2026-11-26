#!/bin/sh
# Generated from '../check-scoped-components.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../check-scoped-components.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../check-scoped-components.ush' --checked-in -o 'check-scoped-components.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'check-scoped-components.ush'
__ush_origin='../check-scoped-components.ush'

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
  -h|--help) printf '%s' 'check-scoped-components.ush

Usage:
  check-scoped-components.ush [--help]
  check-scoped-components.ush [--man [ITEM]]
  check-scoped-components.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
check-scoped-components.ush

SYNOPSIS
check-scoped-components.ush [--help]
check-scoped-components.ush [--man [ITEM]]
check-scoped-components.ush [--complete [PREFIX]]
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

__ush_at 6 'let temp = $env::RUNNER_TEMP'
__ush_env_0=${RUNNER_TEMP-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_6_temp="${__ush_env_0}"
# shellcheck disable=SC2034
temp="${__ush_binding_6_temp}"
__ush_at 7 'let oracle = $"$temp/scoped-oracle"'
__ush_binding_7_oracle="${__ush_binding_6_temp}"'/scoped-oracle'
# shellcheck disable=SC2034
oracle="${__ush_binding_7_oracle}"
__ush_at 8 'let manifest = $"$temp/scoped-components"'
__ush_binding_8_manifest="${__ush_binding_6_temp}"'/scoped-components'
# shellcheck disable=SC2034
manifest="${__ush_binding_8_manifest}"
__ush_at 9 'let styled_session = $"$temp/styled-session.json"'
__ush_binding_9_styled_session="${__ush_binding_6_temp}"'/styled-session.json'
# shellcheck disable=SC2034
styled_session="${__ush_binding_9_styled_session}"
__ush_at 10 'let scoped_session = $"$temp/scoped-session.json"'
__ush_binding_10_scoped_session="${__ush_binding_6_temp}"'/scoped-session.json'
# shellcheck disable=SC2034
scoped_session="${__ush_binding_10_scoped_session}"
__ush_at 11 'let maestro_features = "experimental-lexical-arrow-queries,experimental-recovered-literal-prefix,experimental-scoped-component-session,experimental-style-lint-products,micro-profiling"'
__ush_binding_11_maestro_features='experimental-lexical-arrow-queries,experimental-recovered-literal-prefix,experimental-scoped-component-session,experimental-style-lint-products,micro-profiling'
# shellcheck disable=SC2034
maestro_features="${__ush_binding_11_maestro_features}"
__ush_at 13 'export NODE_PATH = $"$oracle/node_modules"'
export NODE_PATH="${__ush_binding_7_oracle}"'/node_modules'
__ush_at 14 'let sass_version = node (from_source "read-pinned-sass-version.ts")'
__ush_value_1="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__from_source 'read-pinned-sass-version.ts')" || exit "$?"
__ush_value_2="$(__ush_return_path='' __ush_capture_return='1' command node "${__ush_value_1}")" || exit "$?"
__ush_binding_14_sass_version="${__ush_value_2}"
# shellcheck disable=SC2034
sass_version="${__ush_binding_14_sass_version}"
__ush_at 15 'npm install --prefix $oracle --no-audit --no-fund --ignore-scripts --save-exact \'
__ush_value_3="${__ush_binding_14_sass_version}"
command npm 'install' '--prefix' "${__ush_binding_7_oracle}" '--no-audit' '--no-fund' '--ignore-scripts' '--save-exact' 'sass@'"${__ush_value_3}" '@vue/compiler-sfc@3.5.35' '@jridgewell/sourcemap-codec@1.6.0' 'postcss-selector-parser@7.1.6' 'vue@3.5.35'
__ush_at 21 'export VIZE_SCSS_MANIFEST = manifest'
export VIZE_SCSS_MANIFEST="${__ush_binding_8_manifest}"
__ush_at 22 'export VIZE_REQUIRE_REPOSITORY_SCSS = "1"'
export VIZE_REQUIRE_REPOSITORY_SCSS='1'
__ush_at 23 'git ls-files -z -- *.vue *.vue.txt *.scss | to_file $manifest'
__ush_std_fs_path "${__ush_binding_8_manifest}" || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
__ush_path_4=$__ush_path
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (command git 'ls-files' '-z' '--' '*.vue' '*.vue.txt' '*.scss') 7>&- >"$__ush_path_4"; printf ' 1:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: git ls-files -z -- *.vue *.vue.txt *.scss | to_file $mani...' || exit "$?"
__ush_at 24 'cargo test --locked --profile ci-opt -p vize_atelier_sfc \'
command cargo 'test' '--locked' '--profile' 'ci-opt' '-p' 'vize_atelier_sfc' '--features' 'native-scoped-scss' '--test' 'native_sfc_scss' 'observed_scoped_components_match_published_style_and_render_products' '--' '--ignored'
__ush_at 30 'export VIZE_TOOLCHAIN_HISTORY = $"$temp/scoped-revisions"'
export VIZE_TOOLCHAIN_HISTORY="${__ush_binding_6_temp}"'/scoped-revisions'
__ush_at 31 'export VIZE_STYLED_SESSION_OUTPUT = styled_session'
export VIZE_STYLED_SESSION_OUTPUT="${__ush_binding_9_styled_session}"
__ush_at 32 'export VIZE_SCOPED_SESSION_OUTPUT = scoped_session'
export VIZE_SCOPED_SESSION_OUTPUT="${__ush_binding_10_scoped_session}"
__ush_at 33 'cargo test --locked --profile ci-opt -p vize_maestro \'
command cargo 'test' '--locked' '--profile' 'ci-opt' '-p' 'vize_maestro' '--features' "${__ush_binding_11_maestro_features}" '--lib' 'toolchain_session_from_one_open_buffer'
__ush_at 36 'from_file $styled_session | node crates/vize_atelier_sfc/tests/component_scss/oracle.ts'
__ush_std_fs_path "${__ush_binding_9_styled_session}" || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
__ush_path_5=$__ush_path
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (command node 'crates/vize_atelier_sfc/tests/component_scss/oracle.ts') 7>&- <"$__ush_path_5"; printf ' 1:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: from_file $styled_session | node crates/vize_atelier_sfc/...' || exit "$?"
__ush_at 37 'from_file $scoped_session | node crates/vize_atelier_sfc/tests/component_scss/oracle.ts'
__ush_std_fs_path "${__ush_binding_10_scoped_session}" || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
__ush_path_6=$__ush_path
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (command node 'crates/vize_atelier_sfc/tests/component_scss/oracle.ts') 7>&- <"$__ush_path_6"; printf ' 1:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: from_file $scoped_session | node crates/vize_atelier_sfc/...' || exit "$?"
__ush_at 38 'node (from_source "check-style-revisions.ts")'
__ush_value_7="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__from_source 'check-style-revisions.ts')" || exit "$?"
command node "${__ush_value_7}"
__ush_at 39 'cargo test --locked --profile ci-opt -p vize_atelier_sfc \'
command cargo 'test' '--locked' '--profile' 'ci-opt' '-p' 'vize_atelier_sfc' '--features' 'bundler-block-products' '--test' 'bundler_toolchain_session' '--' '--ignored'
__ush_at 43 'cargo test --locked --profile ci-opt \'
command cargo 'test' '--locked' '--profile' 'ci-opt' '--manifest-path' 'crates/vize_incremental/providers/graph/Cargo.toml' '--test' 'incremental' 'original_template_graphs_follow_the_same_toolchain_session' '--' '--ignored'
