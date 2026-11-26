#!/bin/sh
# Generated from '../compare-ambient-diagnostic-revisions.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../compare-ambient-diagnostic-revisions.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../compare-ambient-diagnostic-revisions.ush' --checked-in -o 'compare-ambient-diagnostic-revisions.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'compare-ambient-diagnostic-revisions.ush'
__ush_origin='../compare-ambient-diagnostic-revisions.ush'

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
  -h|--help) printf '%s' 'compare-ambient-diagnostic-revisions.ush

Usage:
  compare-ambient-diagnostic-revisions.ush [--help]
  compare-ambient-diagnostic-revisions.ush [--man [ITEM]]
  compare-ambient-diagnostic-revisions.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
compare-ambient-diagnostic-revisions.ush

SYNOPSIS
compare-ambient-diagnostic-revisions.ush [--help]
compare-ambient-diagnostic-revisions.ush [--man [ITEM]]
compare-ambient-diagnostic-revisions.ush [--complete [PREFIX]]
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

ush_fn_std__fs__is_dir() {
  __ush_path=$(ush_fn_std__path__resolve "$1")
  if [ -d "$__ush_path" ]; then printf '%s' 'true'; else printf '%s' 'false'; fi
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

ush_fn_std__regex__is_match() {
  VALUE="$1" PATTERN="$2" awk 'BEGIN { value = ENVIRON["VALUE"]; pattern = ENVIRON["PATTERN"]; if (value ~ pattern) { printf "true" } else { printf "false" } }'
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

__ush_std_encode_args() {
  for __ush_arg do
    printf x
    printf '%s' "$__ush_arg" | od -A n -v -t x1 | tr -d ' \n'
    printf ' '
  done
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

ush_fn_std__string__lines() {
  [ -n "$1" ] || return 0
  __ush_newline='
'
  __ush_carriage=$(printf '\r')
  __ush_rest=${1%"$__ush_newline"}
  while :; do
    case $__ush_rest in
      *"$__ush_newline"*)
        __ush_line=${__ush_rest%%"$__ush_newline"*}
        __ush_std_encode_args "${__ush_line%"$__ush_carriage"}"
        __ush_rest=${__ush_rest#*"$__ush_newline"} ;;
      *) __ush_std_encode_args "${__ush_rest%"$__ush_carriage"}"; break ;;
    esac
  done
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

__ush_at 10 'let base_sha = $env::BASE_SHA'
__ush_env_0=${BASE_SHA-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable BASE_SHA is not set or is empty'; exit 1; fi
__ush_binding_10_base_sha="${__ush_env_0}"
# shellcheck disable=SC2034
base_sha="${__ush_binding_10_base_sha}"
__ush_at 11 'let head_sha = $env::HEAD_SHA'
__ush_env_1=${HEAD_SHA-}
if [ -z "${__ush_env_1}" ]; then __ush_report 1 'environment variable HEAD_SHA is not set or is empty'; exit 1; fi
__ush_binding_11_head_sha="${__ush_env_1}"
# shellcheck disable=SC2034
head_sha="${__ush_binding_11_head_sha}"
__ush_at 12 'let workspace = $env::GITHUB_WORKSPACE'
__ush_env_2=${GITHUB_WORKSPACE-}
if [ -z "${__ush_env_2}" ]; then __ush_report 1 'environment variable GITHUB_WORKSPACE is not set or is empty'; exit 1; fi
__ush_binding_12_workspace="${__ush_env_2}"
# shellcheck disable=SC2034
workspace="${__ush_binding_12_workspace}"
__ush_at 13 'let temp = $env::RUNNER_TEMP'
__ush_env_3=${RUNNER_TEMP-}
if [ -z "${__ush_env_3}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_13_temp="${__ush_env_3}"
# shellcheck disable=SC2034
temp="${__ush_binding_13_temp}"
__ush_at 14 'let work = $"$temp/ambient-read-checkout"'
__ush_binding_14_work="${__ush_binding_13_temp}"'/ambient-read-checkout'
# shellcheck disable=SC2034
work="${__ush_binding_14_work}"
__ush_at 15 'let suite = "crates/vize_maestro/src/ide/diagnostics/corsa/ambient_project"'
__ush_binding_15_suite='crates/vize_maestro/src/ide/diagnostics/corsa/ambient_project'
# shellcheck disable=SC2034
suite="${__ush_binding_15_suite}"
__ush_at 17 'if !(is_match $base_sha "^[0-9a-f]{40}$") || !(is_match $head_sha "^[0-9a-f]{40}$") {'
if {
! {
__ush_value_4="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__regex__is_match "${__ush_binding_10_base_sha}" '^[0-9a-f]{40}$')" || exit "$?"
[ "${__ush_value_4}" = 'true' ]
} || ! {
__ush_value_5="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__regex__is_match "${__ush_binding_11_head_sha}" '^[0-9a-f]{40}$')" || exit "$?"
[ "${__ush_value_5}" = 'true' ]
}
}; then
  __ush_at 18 'fail 1 "BASE_SHA and HEAD_SHA must be full lowercase commit SHAs"'
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'BASE_SHA and HEAD_SHA must be full lowercase commit SHAs'
  exit "$__ush_status"
fi
__ush_at 20 'git fetch --no-tags --depth=1 origin $base_sha'
command git 'fetch' '--no-tags' '--depth=1' 'origin' "${__ush_binding_10_base_sha}"
__ush_at 21 'export CARGO_TARGET_DIR = $"$workspace/target"'
export CARGO_TARGET_DIR="${__ush_binding_12_workspace}"'/target'
__ush_at 23 'for (side, revision) in [("base", base_sha), ("head", head_sha)] {'
__ush_match_6__len='2'
__ush_match_6__0__len='2'
# shellcheck disable=SC2034
__ush_match_6__0__0='base'
# shellcheck disable=SC2034
__ush_match_6__0__1="${__ush_binding_10_base_sha}"
__ush_match_6__1__len='2'
# shellcheck disable=SC2034
__ush_match_6__1__0='head'
# shellcheck disable=SC2034
__ush_match_6__1__1="${__ush_binding_11_head_sha}"
__ush_for_list_index_7=0
while [ "${__ush_for_list_index_7}" -lt "${__ush_match_6__len}" ]; do
  __ush_for_23__len=0
  ush_fn_std__unpack __ush_for_23 "$(ush_fn_std__pack __ush_match_6__${__ush_for_list_index_7})"
  __ush_for_list_index_7=$((__ush_for_list_index_7 + 1))
  __ush_at 23 'for (side, revision) in [("base", base_sha), ("head", head_sha)] {'
  __ush_binding_23_side="${__ush_for_23__0}"
  # shellcheck disable=SC2034
  side="${__ush_binding_23_side}"
  __ush_at 23 'for (side, revision) in [("base", base_sha), ("head", head_sha)] {'
  __ush_binding_23_revision="${__ush_for_23__1}"
  # shellcheck disable=SC2034
  revision="${__ush_binding_23_revision}"
  __ush_at 25 'if side == "head" {'
  if {
  [ "$(if [ "${__ush_binding_23_side}" = 'head' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
  }; then
    __ush_at 26 'git worktree remove --force $work'
    command git 'worktree' 'remove' '--force' "${__ush_binding_14_work}"
  fi
  __ush_at 28 'git worktree add --quiet --detach $work $revision'
  command git 'worktree' 'add' '--quiet' '--detach' "${__ush_binding_14_work}" "${__ush_binding_23_revision}"
  __ush_at 29 'for dir in ["node_modules", "playground/node_modules"] {'
  __ush_match_11__len='2'
  # shellcheck disable=SC2034
  __ush_match_11__0='node_modules'
  # shellcheck disable=SC2034
  __ush_match_11__1='playground/node_modules'
  __ush_for_list_index_12=0
  while [ "${__ush_for_list_index_12}" -lt "${__ush_match_11__len}" ]; do
    __ush_for_value_13=''
    eval "__ush_for_value_13=\"\${__ush_match_11__${__ush_for_list_index_12}}\""
    dir="${__ush_for_value_13}"
    __ush_for_list_index_12=$((__ush_for_list_index_12 + 1))
    __ush_at 30 'if is_dir $"$workspace/$dir" {'
    if {
    __ush_value_14="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__fs__is_dir "${__ush_binding_12_workspace}"'/'"${dir}")" || exit "$?"
    [ "${__ush_value_14}" = 'true' ]
    }; then
      __ush_at 31 'ln -s $"$workspace/$dir" $"$work/$dir"'
      command ln '-s' "${__ush_binding_12_workspace}"'/'"${dir}" "${__ush_binding_14_work}"'/'"${dir}"
    fi
  done
  __ush_at 36 'if side == "base" {'
  if {
  [ "$(if [ "${__ush_binding_23_side}" = 'base' ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
  }; then
    __ush_at 37 'cp $"$workspace/$suite/tests/cost.rs" $"$work/$suite/tests/cost.rs"'
    command cp "${__ush_binding_12_workspace}"'/'"${__ush_binding_15_suite}"'/tests/cost.rs' "${__ush_binding_14_work}"'/'"${__ush_binding_15_suite}"'/tests/cost.rs'
    __ush_at 38 'if ! grep -Fxq "mod cost;" $"$work/$suite/tests.rs" {'
    if ! {
    command grep '-Fxq' 'mod cost;' "${__ush_binding_14_work}"'/'"${__ush_binding_15_suite}"'/tests.rs'
    }; then
      __ush_at 39 'append_text $"$work/$suite/tests.rs" "mod cost;\n"'
      ( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__append_text "${__ush_binding_14_work}"'/'"${__ush_binding_15_suite}"'/tests.rs' 'mod cost;'"${__ush_lf}"'' )
    fi
  fi
  __ush_at 42 'cmp $"$workspace/$suite/tests/cost.rs" $"$work/$suite/tests/cost.rs"'
  command cmp "${__ush_binding_12_workspace}"'/'"${__ush_binding_15_suite}"'/tests/cost.rs' "${__ush_binding_14_work}"'/'"${__ush_binding_15_suite}"'/tests/cost.rs'
  __ush_at 43 'let build_messages = $"$temp/ambient-$side.jsonl"'
  __ush_binding_43_build_messages="${__ush_binding_13_temp}"'/ambient-'"${__ush_binding_23_side}"'.jsonl'
  # shellcheck disable=SC2034
  build_messages="${__ush_binding_43_build_messages}"
  __ush_at 44 'cd $work {'
  __ush_cwd_15=$PWD
  ush_fn_std__process__chdir "${__ush_binding_14_work}" || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
    __ush_at 45 'cargo test --locked --profile ci-opt -p vize_maestro --lib \'
    __ush_std_fs_path "${__ush_binding_43_build_messages}" || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
    __ush_path_16=$__ush_path
    { __ush_pipe=$(
      exec 7>&1 >&8 8>&-
      set +e
      { (command cargo 'test' '--locked' '--profile' 'ci-opt' '-p' 'vize_maestro' '--lib' '--features' 'experimental-ambient-project,micro-profiling' '--no-run' '--message-format=json') 7>&- >"$__ush_path_16"; printf ' 1:%s' "$?" >&7; }
    ); } 8>&1
    __ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: cargo test --locked --profile ci-opt -p vize_maestro --li...' || exit "$?"
  cd "$__ush_cwd_15" || { __ush_status=$?; __ush_report "$__ush_status" 'cannot return to the directory the cd block started in'; exit "$__ush_status"; }
  __ush_at 51 'let executables = lines (jq -r -f (from_source "native-maestro-test-executable.jq") $build_messages)'
  __ush_value_17="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__from_source 'native-maestro-test-executable.jq')" || exit "$?"
  __ush_value_18="$(__ush_return_path='' __ush_capture_return='1' command jq '-r' '-f' "${__ush_value_17}" "${__ush_binding_43_build_messages}")" || exit "$?"
  __ush_packed_19="$(__ush_return_path='' __ush_capture_return=1 ush_fn_std__string__lines "${__ush_value_18}")" || exit "$?"
  __ush_binding_51_executables__len=0
  ush_fn_std__unpack __ush_binding_51_executables "${__ush_packed_19}"
  __ush_at 52 'if executables.len() != 1 {'
  if {
  __ush_value_20=${__ush_binding_51_executables__len}
  [ "$(if [ "${__ush_value_20}" -ne 1 ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
  }; then
    __ush_at 53 'fail 1 $"The $side build produced ${executables.len()} vize_maestro test executables, not one"'
    __ush_value_21=${__ush_binding_51_executables__len}
    __ush_status=1
    if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
    __ush_report "$__ush_status" 'The '"${__ush_binding_23_side}"' build produced '"${__ush_value_21}"' vize_maestro test executables, not one'
    exit "$__ush_status"
  fi
  __ush_at 55 'cp (executables[0]) $"$temp/ambient-$side"'
  __ush_index_22=0
  if [ "${__ush_index_22}" -lt 0 ] || [ "${__ush_index_22}" -ge "${__ush_binding_51_executables__len}" ]; then __ush_report 1 "index ${__ush_index_22} is out of range for a list of length ${__ush_binding_51_executables__len}"; exit 1; fi
  __ush_item_23=''
  eval "__ush_item_23=\"\${__ush_binding_51_executables__${__ush_index_22}}\""
  command cp "${__ush_item_23}" "${__ush_binding_13_temp}"'/ambient-'"${__ush_binding_23_side}"
done
__ush_at 58 'let head_tests = $"$temp/ambient-head"'
__ush_binding_58_head_tests="${__ush_binding_13_temp}"'/ambient-head'
# shellcheck disable=SC2034
head_tests="${__ush_binding_58_head_tests}"
__ush_at 59 'let phase_test = "ide::diagnostics::corsa::ambient_project::tests::repeated::alternating_hosts_and_raw_reads_keep_the_current_native_answer"'
__ush_binding_59_phase_test='ide::diagnostics::corsa::ambient_project::tests::repeated::alternating_hosts_and_raw_reads_keep_the_current_native_answer'
# shellcheck disable=SC2034
phase_test="${__ush_binding_59_phase_test}"
__ush_at 60 'exec $head_tests ["--list"] | grep -Fx $"$phase_test: test"'
__ush_match_24__len='1'
# shellcheck disable=SC2034
__ush_match_24__0='--list'
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (ush_fn_std__process__exec "${__ush_binding_58_head_tests}" "$(ush_fn_std__pack __ush_match_24)") 7>&-; printf ' 1:%s' "$?" >&7; } |
  { (command grep '-Fx' "${__ush_binding_59_phase_test}"': test') 7>&-; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: exec $head_tests ["--list"]' 'pipeline stage 2 failed: grep -Fx $"$phase_test: test"' || exit "$?"
__ush_at 61 'exec $head_tests ["--exact", "--ignored", phase_test]'
__ush_match_26__len='3'
# shellcheck disable=SC2034
__ush_match_26__0='--exact'
# shellcheck disable=SC2034
__ush_match_26__1='--ignored'
# shellcheck disable=SC2034
__ush_match_26__2="${__ush_binding_59_phase_test}"
( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__process__exec "${__ush_binding_58_head_tests}" "$(ush_fn_std__pack __ush_match_26)" )
__ush_at 63 'node tools/benchmarks/scripts/compare-ambient-diagnostics.ts \'
__ush_env_28=${GITHUB_STEP_SUMMARY-}
if [ -z "${__ush_env_28}" ]; then __ush_report 1 'environment variable GITHUB_STEP_SUMMARY is not set or is empty'; exit 1; fi
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (command node 'tools/benchmarks/scripts/compare-ambient-diagnostics.ts' "${__ush_binding_13_temp}"'/ambient-base' "${__ush_binding_58_head_tests}" "${__ush_binding_13_temp}"'/ambient-diagnostics.json') 7>&-; printf ' 1:%s' "$?" >&7; } |
  { (command tee '-a' "${__ush_env_28}") 7>&-; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: node tools/benchmarks/scripts/compare-ambient-diagnostics...' 'pipeline stage 2 failed: tee -a $env::GITHUB_STEP_SUMMARY' || exit "$?"
