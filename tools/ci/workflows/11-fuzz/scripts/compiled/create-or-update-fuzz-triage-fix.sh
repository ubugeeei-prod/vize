#!/bin/sh
# Generated from '../create-or-update-fuzz-triage-fix.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../create-or-update-fuzz-triage-fix.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../create-or-update-fuzz-triage-fix.ush' --checked-in -o 'create-or-update-fuzz-triage-fix.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'create-or-update-fuzz-triage-fix.ush'
__ush_origin='../create-or-update-fuzz-triage-fix.ush'

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

__ush_jobs=''
__ush_signal=''
__ush_task_seq=0
__ush_task_dir=''
__ush_task_init() {
  [ -z "$__ush_task_dir" ] || return 0
  __ush_task_dir=$(
    umask 077
    CDPATH='' cd -P "${TMPDIR:-/tmp}" || exit 1
    __ush_base=$(pwd -P)
    __ush_n=0
    while [ "$__ush_n" -lt 1000 ]; do
      __ush_dir=$__ush_base/ush-tasks.$$.$__ush_n
      if mkdir "$__ush_dir" 2>/dev/null; then printf '%s' "$__ush_dir"; exit 0; fi
      __ush_n=$((__ush_n + 1))
    done
    printf '%s\n' 'ush: cannot create private task directory' >&2
    exit 1
  )
}

__ush_task_slot() {
  while :; do
    __ush_task_seq=$((__ush_task_seq + 1))
    __ush_slot=$__ush_task_dir/$__ush_task_seq
    if mkdir "$__ush_slot" 2>/dev/null; then break; fi
    [ -d "$__ush_slot" ] || return 1
  done
}
__ush_wait_job() {
  __ush_wait_fallback=127
  case " $__ush_jobs " in
    *" $1 "*)
      __ush_wait_fallback=0
      wait "$1" 2>/dev/null || __ush_wait_fallback=$?
      # A task a signal ended wrote no status, or, in some shells, the
      # status of its last command; keep the one wait saw, so every later
      # await observes it too.
      if [ "$__ush_wait_fallback" -gt 128 ] || [ ! -s "$2" ]; then
        printf '%s\n' "$__ush_wait_fallback" > "$2"
      fi
      __ush_remaining=''
      for __ush_pid in $__ush_jobs; do
        if [ "$__ush_pid" != "$1" ]; then __ush_remaining="$__ush_remaining $__ush_pid"; fi
      done
      __ush_jobs=$__ush_remaining ;;
  esac
  # Only the launching shell can reap the child. Other task scopes may
  # also await its result, including after the launching shell has reaped it.
  while ! IFS= read -r __ush_wait_status < "$2"; do
    if ! kill -0 "$1" 2>/dev/null; then
      IFS= read -r __ush_wait_status < "$2" || return "$__ush_wait_fallback"
      break
    fi
    sleep 0.01 2>/dev/null || sleep 1
  done
  return "$__ush_wait_status"
}
# Report how the scope ended, run its deferred blocks, then wait for its
# unfinished tasks. A task killed through its handle does not fail the scope.
__ush_join() {
  __ush_exit=$1
  __ush_report "$__ush_exit"
  [ -z "${__ush_defers:-}" ] || __ush_run_defers
  __ush_line=''
  for __ush_pid in $__ush_jobs; do
    case " ${__ush_killed:-} " in
      *" $__ush_pid "*) wait "$__ush_pid" 2>/dev/null || : ;;
      *) wait "$__ush_pid" || { __ush_child=$?; [ "$__ush_exit" -ne 0 ] || __ush_exit=$__ush_child; } ;;
    esac
  done
}
# A task or function scope. It leaves signals alone, so a task killed
# through its handle dies at once even before it starts its command; a shell
# would otherwise hold a trapped signal until its foreground command ends.
__ush_task_scope() {
  __ush_jobs=''
  __ush_defers=''
  __ush_completion=${1:-}
  trap '__ush_end_scope "$?"' 0
  # A task's handle may now signal it: the shell has dropped the traps it
  # inherited, which would otherwise swallow the signal.
  [ -z "$__ush_completion" ] || : > "$__ush_completion.started"
}
__ush_end_scope() {
  __ush_join "$1"
  if [ -n "$__ush_completion" ]; then
    printf '%s\n' "$__ush_exit" > "$__ush_completion"
  fi
  exit "$__ush_exit"
}
__ush_finish() {
  __ush_join "$1"
  [ -z "$__ush_task_dir" ] || rm -rf "$__ush_task_dir"
  exit "$__ush_exit"
}
trap '__ush_finish "${__ush_signal:-$?}"' 0
trap '__ush_signal=129; exit 129' HUP
trap '__ush_signal=130; exit 130' INT
trap '__ush_signal=143; exit 143' TERM

case "${1:-}" in
  -h|--help) printf '%s' 'create-or-update-fuzz-triage-fix.ush

Usage:
  create-or-update-fuzz-triage-fix.ush [--help]
  create-or-update-fuzz-triage-fix.ush [--man [ITEM]]
  create-or-update-fuzz-triage-fix.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
create-or-update-fuzz-triage-fix.ush

SYNOPSIS
create-or-update-fuzz-triage-fix.ush [--help]
create-or-update-fuzz-triage-fix.ush [--man [ITEM]]
create-or-update-fuzz-triage-fix.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

exec 3>&1

ush_fn_std__fs__read_text() {
  __ush_path=$(ush_fn_std__path__resolve "$1")
  cat "$__ush_path"
}

ush_fn_std__fs__write_text() {
  __ush_path=$(ush_fn_std__path__resolve "$1")
  printf '%s' "$2" > "$__ush_path"
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

ush_fn_std__path__basename() {
  basename "$(__ush_std_path_resolve "$1")"
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

ush_fn_std__fs__temp_dir() {
  mktemp -d "$__ush_task_dir/tmp.XXXXXXXXXX"
}

ush_fn_std__string__is_empty() {
  if [ -z "$1" ]; then printf true; else printf false; fi
}

__ush_std_fs_path() {
  __ush_path=$(ush_fn_std__path__resolve "$1" && printf .) || return "$?"
  __ush_path=${__ush_path%.}
  [ -n "$__ush_path" ] || { printf '%s\n' 'std::fs: path must not be empty' >&2; return 1; }
  case $__ush_path in .|..|/*|./*|../*) ;; *) __ush_path=./$__ush_path ;; esac
}

ush_fn_std__fs__read_dir() {
  __ush_std_fs_path "$1" || return "$?"
  __ush_dir=$__ush_path
  [ -d "$__ush_dir" ] && [ -r "$__ush_dir" ] && [ -x "$__ush_dir" ] || {
    printf 'std::fs::read_dir: cannot read directory: %s\n' "$__ush_dir" >&2
    return 1
  }
  for __ush_entry in "$__ush_dir"/* "$__ush_dir"/.[!.]* "$__ush_dir"/..?*; do
    if [ -e "$__ush_entry" ] || [ -L "$__ush_entry" ]; then
      __ush_std_encode_args "$__ush_entry"
    fi
  done
}

__ush_std_list_contains() {
  eval "__ush_list_len=\${${1}__len}"
  __ush_list_index=0
  while [ "$__ush_list_index" -lt "$__ush_list_len" ]; do
    eval "__ush_list_item=\${${1}__${__ush_list_index}}"
    if [ "$__ush_list_item" = "$2" ]; then return 0; fi
    __ush_list_index=$((__ush_list_index + 1))
  done
  return 1
}

ush_fn_artifact_kind() {
  __ush_fn_artifact_kind_arg_0="${1}"
  # shellcheck disable=SC2034
  name="${__ush_fn_artifact_kind_arg_0}"
  __ush_at 9 'match name {'
  case "${__ush_fn_artifact_kind_arg_0}" in
    'crash-'*)
      __ush_at 10 'glob "crash-*" => "crash reproducer"'
      if [ -n "${__ush_return_path:-}" ]; then
        printf '%s' 'crash reproducer' > "$__ush_return_path"
      elif [ "${__ush_capture_return:-0}" = '1' ]; then
        printf '%s' 'crash reproducer' >&4
      fi
      return 0
      ;;
    'timeout-'*)
      __ush_at 11 'glob "timeout-*" => "timeout reproducer"'
      if [ -n "${__ush_return_path:-}" ]; then
        printf '%s' 'timeout reproducer' > "$__ush_return_path"
      elif [ "${__ush_capture_return:-0}" = '1' ]; then
        printf '%s' 'timeout reproducer' >&4
      fi
      return 0
      ;;
    'oom-'*)
      __ush_at 12 'glob "oom-*" => "OOM reproducer"'
      if [ -n "${__ush_return_path:-}" ]; then
        printf '%s' 'OOM reproducer' > "$__ush_return_path"
      elif [ "${__ush_capture_return:-0}" = '1' ]; then
        printf '%s' 'OOM reproducer' >&4
      fi
      return 0
      ;;
    'leak-'*)
      __ush_at 13 'glob "leak-*" => "leak reproducer"'
      if [ -n "${__ush_return_path:-}" ]; then
        printf '%s' 'leak reproducer' > "$__ush_return_path"
      elif [ "${__ush_capture_return:-0}" = '1' ]; then
        printf '%s' 'leak reproducer' >&4
      fi
      return 0
      ;;
    'slow-unit-'*)
      __ush_at 14 'glob "slow-unit-*" => "slow unit"'
      if [ -n "${__ush_return_path:-}" ]; then
        printf '%s' 'slow unit' > "$__ush_return_path"
      elif [ "${__ush_capture_return:-0}" = '1' ]; then
        printf '%s' 'slow unit' >&4
      fi
      return 0
      ;;
    *)
      __ush_at 15 '_ => ""'
      if [ -n "${__ush_return_path:-}" ]; then
        printf '%s' '' > "$__ush_return_path"
      elif [ "${__ush_capture_return:-0}" = '1' ]; then
        printf '%s' '' >&4
      fi
      return 0
      ;;
  esac
}

__ush_at 19 'let target = $env::VIZE_MATRIX_TARGET'
__ush_env_0=${VIZE_MATRIX_TARGET-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable VIZE_MATRIX_TARGET is not set or is empty'; exit 1; fi
__ush_binding_19_target="${__ush_env_0}"
# shellcheck disable=SC2034
target="${__ush_binding_19_target}"
__ush_at 20 'let title = $"fix(fuzz): $target found a reproducer"'
__ush_binding_20_title='fix(fuzz): '"${__ush_binding_19_target}"' found a reproducer'
# shellcheck disable=SC2034
title="${__ush_binding_20_title}"
__ush_at 21 'let run_url = $"$env::GITHUB_SERVER_URL/$env::GITHUB_REPOSITORY/actions/runs/$env::GITHUB_RUN_ID"'
__ush_env_1=${GITHUB_SERVER_URL-}
if [ -z "${__ush_env_1}" ]; then __ush_report 1 'environment variable GITHUB_SERVER_URL is not set or is empty'; exit 1; fi
__ush_env_2=${GITHUB_REPOSITORY-}
if [ -z "${__ush_env_2}" ]; then __ush_report 1 'environment variable GITHUB_REPOSITORY is not set or is empty'; exit 1; fi
__ush_env_3=${GITHUB_RUN_ID-}
if [ -z "${__ush_env_3}" ]; then __ush_report 1 'environment variable GITHUB_RUN_ID is not set or is empty'; exit 1; fi
__ush_binding_21_run_url="${__ush_env_1}"'/'"${__ush_env_2}"'/actions/runs/'"${__ush_env_3}"
# shellcheck disable=SC2034
run_url="${__ush_binding_21_run_url}"
__ush_at 22 'let artifact_dir = $"tests/fuzz/artifacts/$target"'
__ush_binding_22_artifact_dir='tests/fuzz/artifacts/'"${__ush_binding_19_target}"
# shellcheck disable=SC2034
artifact_dir="${__ush_binding_22_artifact_dir}"
__ush_at 26 'let mut reproducer_kind = "reproducer artifact"'
__ush_binding_26_reproducer_kind='reproducer artifact'
# shellcheck disable=SC2034
reproducer_kind="${__ush_binding_26_reproducer_kind}"
__ush_at 27 'if is_dir $artifact_dir {'
__ush_task_init
__ush_at 27 'if is_dir $artifact_dir {'
if {
__ush_value_4="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__fs__is_dir "${__ush_binding_22_artifact_dir}")" || exit "$?"
[ "${__ush_value_4}" = 'true' ]
}; then
  __ush_at 28 'let found = [artifact_kind (basename $file) for file in read_dir $artifact_dir]'
  __ush_comprehension_count_5=0
  __ush_packed_7="$(__ush_return_path='' __ush_capture_return=1 ush_fn_std__fs__read_dir "${__ush_binding_22_artifact_dir}")" || exit "$?"
  __ush_match_6__len=0
  ush_fn_std__unpack __ush_match_6 "${__ush_packed_7}"
  __ush_comprehension_index_8=0
  while [ "${__ush_comprehension_index_8}" -lt "${__ush_match_6__len}" ]; do
  eval "__ush_comprehension_item_9=\"\${__ush_match_6__${__ush_comprehension_index_8}}\""
  __ush_value_11="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__basename "${__ush_comprehension_item_9}")" || exit "$?"
  __ush_value_12="$(__ush_return_path='' __ush_capture_return='1' ush_fn_artifact_kind "${__ush_value_11}" 4>&1 >&3)" || exit "$?"
  # shellcheck disable=SC2034
  __ush_comprehension_value_10="${__ush_value_12}"
  eval "__ush_binding_28_found__${__ush_comprehension_count_5}=\$__ush_comprehension_value_10"
  __ush_comprehension_count_5=$((__ush_comprehension_count_5 + 1))
  __ush_comprehension_index_8=$((__ush_comprehension_index_8 + 1))
  done
  __ush_binding_28_found__len="${__ush_comprehension_count_5}"
  __ush_at 30 'let by_severity = ["crash reproducer", "timeout reproducer", "OOM reproducer", "leak reproducer", "slow unit"]'
  __ush_binding_30_by_severity__len='5'
  # shellcheck disable=SC2034
  __ush_binding_30_by_severity__0='crash reproducer'
  # shellcheck disable=SC2034
  __ush_binding_30_by_severity__1='timeout reproducer'
  # shellcheck disable=SC2034
  __ush_binding_30_by_severity__2='OOM reproducer'
  # shellcheck disable=SC2034
  __ush_binding_30_by_severity__3='leak reproducer'
  # shellcheck disable=SC2034
  __ush_binding_30_by_severity__4='slow unit'
  __ush_at 31 'let present = [kind for kind in by_severity if found.contains(kind)]'
  __ush_comprehension_count_13=0
  __ush_match_14__len="${__ush_binding_30_by_severity__len}"
  __ush_copy_index_15=0
  while [ "$__ush_copy_index_15" -lt "${__ush_binding_30_by_severity__len}" ]; do
    eval "__ush_copy_value_16=\"\${__ush_binding_30_by_severity__$__ush_copy_index_15}\""
    eval "__ush_match_14__${__ush_copy_index_15}=\$__ush_copy_value_16"
    __ush_copy_index_15=$((__ush_copy_index_15 + 1))
  done
  __ush_comprehension_index_17=0
  while [ "${__ush_comprehension_index_17}" -lt "${__ush_match_14__len}" ]; do
  eval "__ush_comprehension_item_18=\"\${__ush_match_14__${__ush_comprehension_index_17}}\""
  if __ush_std_list_contains __ush_binding_28_found "${__ush_comprehension_item_18}"; then __ush_value_19=true; else __ush_value_19=false; fi
  if [ "${__ush_value_19}" = true ]; then
  # shellcheck disable=SC2034
  __ush_comprehension_value_20="${__ush_comprehension_item_18}"
  eval "__ush_binding_31_present__${__ush_comprehension_count_13}=\$__ush_comprehension_value_20"
  __ush_comprehension_count_13=$((__ush_comprehension_count_13 + 1))
  fi
  __ush_comprehension_index_17=$((__ush_comprehension_index_17 + 1))
  done
  __ush_binding_31_present__len="${__ush_comprehension_count_13}"
  __ush_at 32 'if !present.is_empty() {'
  if ! {
  if [ "${__ush_binding_31_present__len}" -eq 0 ]; then __ush_value_21=true; else __ush_value_21=false; fi
  [ "${__ush_value_21}" = 'true' ]
  }; then
    __ush_at 33 'reproducer_kind = present.first()'
    if [ "${__ush_binding_31_present__len}" -eq 0 ]; then __ush_report 1 'first() needs a nonempty list'; exit 1; fi
    __ush_index_22=0
    __ush_item_23=''
    eval "__ush_item_23=\"\${__ush_binding_31_present__${__ush_index_22}}\""
    __ush_binding_26_reproducer_kind="${__ush_item_23}"
    # shellcheck disable=SC2034
    reproducer_kind="${__ush_binding_26_reproducer_kind}"
  fi
fi
__ush_at 37 'let body = $"""'
__ush_env_24=${GITHUB_SHA-}
if [ -z "${__ush_env_24}" ]; then __ush_report 1 'environment variable GITHUB_SHA is not set or is empty'; exit 1; fi
__ush_binding_37_body='Fuzz target `'"${__ush_binding_19_target}"'` produced a '"${__ush_binding_26_reproducer_kind}"'.'"${__ush_lf}""${__ush_lf}"'- Workflow run: '"${__ush_binding_21_run_url}""${__ush_lf}"'- Artifact: `fuzz-reproducers-'"${__ush_binding_19_target}"'`'"${__ush_lf}"'- Commit: `'"${__ush_env_24}"'`'"${__ush_lf}""${__ush_lf}"'Download the artifact from the workflow run and replay it locally with:'"${__ush_lf}""${__ush_lf}"'```sh'"${__ush_lf}"'cd tests/fuzz'"${__ush_lf}"'cargo +nightly fuzz run '"${__ush_binding_19_target}"' artifacts/'"${__ush_binding_19_target}"'/<reproducer>'"${__ush_lf}"'```'
# shellcheck disable=SC2034
body="${__ush_binding_37_body}"
__ush_at 51 'let scratch = temp_dir'
__ush_value_25="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__fs__temp_dir)" || exit "$?"
__ush_binding_51_scratch="${__ush_value_25}"
# shellcheck disable=SC2034
scratch="${__ush_binding_51_scratch}"
__ush_at 52 'let body_file = $"$scratch/body.md"'
__ush_binding_52_body_file="${__ush_binding_51_scratch}"'/body.md'
# shellcheck disable=SC2034
body_file="${__ush_binding_52_body_file}"
__ush_at 53 'write_text $body_file $"$body\n"'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__write_text "${__ush_binding_52_body_file}" "${__ush_binding_37_body}""${__ush_lf}" )
__ush_at 55 'let existing_fix = gh issue list \'
__ush_value_26="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__from_source 'first-pr-number.jq')" || exit "$?"
__ush_value_27="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__fs__read_text "${__ush_value_26}")" || exit "$?"
__ush_value_28="$(__ush_return_path='' __ush_capture_return='1' command gh 'issue' 'list' '--state' 'open' '--search' "${__ush_binding_20_title}"' in:title' '--json' 'number' '--jq' "${__ush_value_27}")" || exit "$?"
__ush_binding_55_existing_fix="${__ush_value_28}"
# shellcheck disable=SC2034
existing_fix="${__ush_binding_55_existing_fix}"
__ush_at 61 'if is_empty $existing_fix {'
if {
__ush_value_29="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__string__is_empty "${__ush_binding_55_existing_fix}")" || exit "$?"
[ "${__ush_value_29}" = 'true' ]
}; then
  __ush_at 62 'gh issue create --title $title --body-file $body_file'
  command gh 'issue' 'create' '--title' "${__ush_binding_20_title}" '--body-file' "${__ush_binding_52_body_file}"
else
  __ush_at 64 'gh issue comment $existing_fix --body-file $body_file'
  command gh 'issue' 'comment' "${__ush_binding_55_existing_fix}" '--body-file' "${__ush_binding_52_body_file}"
fi
