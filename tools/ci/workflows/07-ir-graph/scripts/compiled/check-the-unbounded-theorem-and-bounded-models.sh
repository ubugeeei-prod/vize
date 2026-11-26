#!/bin/sh
# Generated from '../check-the-unbounded-theorem-and-bounded-models.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../check-the-unbounded-theorem-and-bounded-models.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../check-the-unbounded-theorem-and-bounded-models.ush' --checked-in -o 'check-the-unbounded-theorem-and-bounded-models.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'check-the-unbounded-theorem-and-bounded-models.ush'
__ush_origin='../check-the-unbounded-theorem-and-bounded-models.ush'

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
  -h|--help) printf '%s' 'check-the-unbounded-theorem-and-bounded-models.ush

Usage:
  check-the-unbounded-theorem-and-bounded-models.ush [--help]
  check-the-unbounded-theorem-and-bounded-models.ush [--man [ITEM]]
  check-the-unbounded-theorem-and-bounded-models.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
check-the-unbounded-theorem-and-bounded-models.ush

SYNOPSIS
check-the-unbounded-theorem-and-bounded-models.ush [--help]
check-the-unbounded-theorem-and-bounded-models.ush [--man [ITEM]]
check-the-unbounded-theorem-and-bounded-models.ush [--complete [PREFIX]]
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

__ush_std_fs_path() {
  __ush_path=$(ush_fn_std__path__resolve "$1" && printf .) || return "$?"
  __ush_path=${__ush_path%.}
  [ -n "$__ush_path" ] || { printf '%s\n' 'std::fs: path must not be empty' >&2; return 1; }
  case $__ush_path in .|..|/*|./*|../*) ;; *) __ush_path=./$__ush_path ;; esac
}

# raises: unknown
ush_fn_alloy() {
  __ush_fn_alloy_arg_0="${1}"
  # shellcheck disable=SC2034
  jar="${__ush_fn_alloy_arg_0}"
  __ush_fn_alloy_arg_1="${2}"
  # shellcheck disable=SC2034
  instances="${__ush_fn_alloy_arg_1}"
  __ush_fn_alloy_arg_2="${3}"
  # shellcheck disable=SC2034
  model="${__ush_fn_alloy_arg_2}"
  __ush_at 9 'java -Djava.awt.headless=true -jar $jar exec -q -c "*" -t none -o $instances $model'
  command java '-Djava.awt.headless=true' '-jar' "${__ush_fn_alloy_arg_0}" 'exec' '-q' '-c' '*' '-t' 'none' '-o' "${__ush_fn_alloy_arg_1}" "${__ush_fn_alloy_arg_2}" || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
}

# raises: unknown
ush_fn_tlc() {
  __ush_fn_tlc_arg_0="${1}"
  # shellcheck disable=SC2034
  jar="${__ush_fn_tlc_arg_0}"
  __ush_fn_tlc_arg_1="${2}"
  # shellcheck disable=SC2034
  metadir="${__ush_fn_tlc_arg_1}"
  __ush_fn_tlc_arg_2="${3}"
  # shellcheck disable=SC2034
  config="${__ush_fn_tlc_arg_2}"
  __ush_fn_tlc_arg_3="${4}"
  # shellcheck disable=SC2034
  spec="${__ush_fn_tlc_arg_3}"
  __ush_at 14 'java -cp $jar tlc2.TLC -workers 1 -metadir $metadir -config $config $spec'
  command java '-cp' "${__ush_fn_tlc_arg_0}" 'tlc2.TLC' '-workers' 1 '-metadir' "${__ush_fn_tlc_arg_1}" '-config' "${__ush_fn_tlc_arg_2}" "${__ush_fn_tlc_arg_3}" || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
}

# raises: unknown
ush_fn_require_violation() {
  __ush_fn_require_violation_arg_0="${1}"
  # shellcheck disable=SC2034
  jar="${__ush_fn_require_violation_arg_0}"
  __ush_fn_require_violation_arg_1="${2}"
  # shellcheck disable=SC2034
  metadir="${__ush_fn_require_violation_arg_1}"
  __ush_fn_require_violation_arg_2="${3}"
  # shellcheck disable=SC2034
  config="${__ush_fn_require_violation_arg_2}"
  __ush_fn_require_violation_arg_3="${4}"
  # shellcheck disable=SC2034
  spec="${__ush_fn_require_violation_arg_3}"
  __ush_fn_require_violation_arg_4="${5}"
  # shellcheck disable=SC2034
  invariant="${__ush_fn_require_violation_arg_4}"
  __ush_fn_require_violation_arg_5="${6}"
  # shellcheck disable=SC2034
  log="${__ush_fn_require_violation_arg_5}"
  __ush_at 20 'let result = status_of tlc $jar $metadir $config $spec |& to_file $log'
  __ush_std_fs_path "${__ush_fn_require_violation_arg_5}" || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
  __ush_path_0=$__ush_path
  if ( __ush_capture_return='0' __ush_return_path=''; ush_fn_tlc "${__ush_fn_require_violation_arg_0}" "${__ush_fn_require_violation_arg_1}" "${__ush_fn_require_violation_arg_2}" "${__ush_fn_require_violation_arg_3}" ) >"$__ush_path_0" 2>&1; then __ush_status_1=0; else __ush_status_1=$?; fi
  __ush_binding_20_result="${__ush_status_1}"
  # shellcheck disable=SC2034
  result="${__ush_binding_20_result}"
  __ush_at 21 'if result != 12 {'
  if {
  [ "$(if [ "${__ush_binding_20_result}" -ne 12 ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
  }; then
    __ush_at 22 'fail 1 $"TLC exited $result, not 12, with $config"'
    __ush_status=1
    if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
    __ush_report "$__ush_status" 'TLC exited '"${__ush_binding_20_result}"', not 12, with '"${__ush_fn_require_violation_arg_2}"
    return "$__ush_status"
  fi
  __ush_at 24 'grep -F $"Invariant $invariant is violated" $log'
  command grep '-F' 'Invariant '"${__ush_fn_require_violation_arg_4}"' is violated' "${__ush_fn_require_violation_arg_5}" || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
}

__ush_at 27 'let temp = $env::RUNNER_TEMP'
__ush_env_2=${RUNNER_TEMP-}
if [ -z "${__ush_env_2}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_27_temp="${__ush_env_2}"
# shellcheck disable=SC2034
temp="${__ush_binding_27_temp}"
__ush_at 28 'let lean = $"$temp/lean-4.33.1-linux/bin/lean"'
__ush_binding_28_lean="${__ush_binding_27_temp}"'/lean-4.33.1-linux/bin/lean'
# shellcheck disable=SC2034
lean="${__ush_binding_28_lean}"
__ush_at 29 'let alloy_jar = $"$temp/alloy.jar"'
__ush_binding_29_alloy_jar="${__ush_binding_27_temp}"'/alloy.jar'
# shellcheck disable=SC2034
alloy_jar="${__ush_binding_29_alloy_jar}"
__ush_at 30 'let tlc_jar = $"$temp/tlc.jar"'
__ush_binding_30_tlc_jar="${__ush_binding_27_temp}"'/tlc.jar'
# shellcheck disable=SC2034
tlc_jar="${__ush_binding_30_tlc_jar}"
__ush_at 31 'let theorem = $"$temp/theorem"'
__ush_binding_31_theorem="${__ush_binding_27_temp}"'/theorem'
# shellcheck disable=SC2034
theorem="${__ush_binding_31_theorem}"
__ush_at 32 'let positions = $"$temp/positions"'
__ush_binding_32_positions="${__ush_binding_27_temp}"'/positions'
# shellcheck disable=SC2034
positions="${__ush_binding_32_positions}"
__ush_at 34 'exec $lean ["-DwarningAsError=true", "Reach.lean"] | tee $theorem'
__ush_match_3__len='2'
# shellcheck disable=SC2034
__ush_match_3__0='-DwarningAsError=true'
# shellcheck disable=SC2034
__ush_match_3__1='Reach.lean'
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (ush_fn_std__process__exec "${__ush_binding_28_lean}" "$(ush_fn_std__pack __ush_match_3)") 7>&-; printf ' 1:%s' "$?" >&7; } |
  { (command tee "${__ush_binding_31_theorem}") 7>&-; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: exec $lean ["-DwarningAsError=true", "Reach.lean"]' 'pipeline stage 2 failed: tee $theorem' || exit "$?"
__ush_at 35 'grep -Fx "'"'"'Davinci.exact_answer'"'"' does not depend on any axioms" $theorem'
command grep '-Fx' ''"'"'Davinci.exact_answer'"'"' does not depend on any axioms' "${__ush_binding_31_theorem}"
__ush_at 36 'exec $lean ["-DwarningAsError=true", "../position-index/Intervals.lean"] | to_file $positions'
__ush_match_5__len='2'
# shellcheck disable=SC2034
__ush_match_5__0='-DwarningAsError=true'
# shellcheck disable=SC2034
__ush_match_5__1='../position-index/Intervals.lean'
__ush_std_fs_path "${__ush_binding_32_positions}" || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
__ush_path_7=$__ush_path
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (ush_fn_std__process__exec "${__ush_binding_28_lean}" "$(ush_fn_std__pack __ush_match_5)") 7>&- >"$__ush_path_7"; printf ' 1:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: exec $lean ["-DwarningAsError=true", "../position-index/I...' || exit "$?"
__ush_at 37 'grep -Fx "'"'"'Davinci.PositionIndex.point_eq_scan'"'"' depends on axioms: [propext, Classical.choice, Quot.sound]" $positions'
command grep '-Fx' ''"'"'Davinci.PositionIndex.point_eq_scan'"'"' depends on axioms: [propext, Classical.choice, Quot.sound]' "${__ush_binding_32_positions}"
__ush_at 38 'grep -Fx "'"'"'Davinci.PositionIndex.selected_rows'"'"' depends on axioms: [propext, Classical.choice, Quot.sound]" $positions'
command grep '-Fx' ''"'"'Davinci.PositionIndex.selected_rows'"'"' depends on axioms: [propext, Classical.choice, Quot.sound]' "${__ush_binding_32_positions}"
__ush_at 39 'grep -Fx "'"'"'Davinci.authored_slice'"'"' depends on axioms: [propext, Quot.sound]" $theorem'
command grep '-Fx' ''"'"'Davinci.authored_slice'"'"' depends on axioms: [propext, Quot.sound]' "${__ush_binding_31_theorem}"
__ush_at 40 'grep -Fx "'"'"'Davinci.recovery_sites_exact'"'"' depends on axioms: [propext, Quot.sound]" $theorem'
command grep '-Fx' ''"'"'Davinci.recovery_sites_exact'"'"' depends on axioms: [propext, Quot.sound]' "${__ush_binding_31_theorem}"
__ush_at 41 'alloy $alloy_jar $"$temp/instance" Reach.als'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_alloy "${__ush_binding_29_alloy_jar}" "${__ush_binding_27_temp}"'/instance' 'Reach.als' )
__ush_at 42 'jq -e -f (from_source "check-directed-graph.jq") $"$temp/instance/receipt.json"'
__ush_value_8="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__from_source 'check-directed-graph.jq')" || exit "$?"
command jq '-e' '-f' "${__ush_value_8}" "${__ush_binding_27_temp}"'/instance/receipt.json'
__ush_at 43 'grep -Fx "'"'"'Davinci.exact_trails'"'"' depends on axioms: [propext, Classical.choice, Quot.sound]" $theorem'
command grep '-Fx' ''"'"'Davinci.exact_trails'"'"' depends on axioms: [propext, Classical.choice, Quot.sound]' "${__ush_binding_31_theorem}"
__ush_at 44 'alloy $alloy_jar $"$temp/trail-instances" Trail.als'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_alloy "${__ush_binding_29_alloy_jar}" "${__ush_binding_27_temp}"'/trail-instances' 'Trail.als' )
__ush_at 45 'jq -e -f (from_source "check-edge-trails.jq") $"$temp/trail-instances/receipt.json"'
__ush_value_9="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__from_source 'check-edge-trails.jq')" || exit "$?"
command jq '-e' '-f' "${__ush_value_9}" "${__ush_binding_27_temp}"'/trail-instances/receipt.json'
__ush_at 47 'java -cp $tlc_jar tlc2.TLC -workers 2 -metadir $"$temp/tlc-ok" -config Publication.cfg Publication.tla'
command java '-cp' "${__ush_binding_30_tlc_jar}" 'tlc2.TLC' '-workers' 2 '-metadir' "${__ush_binding_27_temp}"'/tlc-ok' '-config' 'Publication.cfg' 'Publication.tla'
__ush_at 48 'require_violation $tlc_jar $"$temp/tlc-negative" Stale.cfg Publication.tla CurrentAnswer $"$temp/stale"'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_require_violation "${__ush_binding_30_tlc_jar}" "${__ush_binding_27_temp}"'/tlc-negative' 'Stale.cfg' 'Publication.tla' 'CurrentAnswer' "${__ush_binding_27_temp}"'/stale' )
__ush_at 49 'tlc $tlc_jar $"$temp/request-model" Request.cfg Request.tla'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_tlc "${__ush_binding_30_tlc_jar}" "${__ush_binding_27_temp}"'/request-model' 'Request.cfg' 'Request.tla' )
__ush_at 50 'for (config, invariant) in [("Version", "CurrentRevision"), ("Cancelled", "LiveRequest"), ("Partial", "NoPartialOutput")] {'
__ush_match_10__len='3'
__ush_match_10__0__len='2'
# shellcheck disable=SC2034
__ush_match_10__0__0='Version'
# shellcheck disable=SC2034
__ush_match_10__0__1='CurrentRevision'
__ush_match_10__1__len='2'
# shellcheck disable=SC2034
__ush_match_10__1__0='Cancelled'
# shellcheck disable=SC2034
__ush_match_10__1__1='LiveRequest'
__ush_match_10__2__len='2'
# shellcheck disable=SC2034
__ush_match_10__2__0='Partial'
# shellcheck disable=SC2034
__ush_match_10__2__1='NoPartialOutput'
__ush_for_list_index_11=0
while [ "${__ush_for_list_index_11}" -lt "${__ush_match_10__len}" ]; do
  __ush_for_50__len=0
  ush_fn_std__unpack __ush_for_50 "$(ush_fn_std__pack __ush_match_10__${__ush_for_list_index_11})"
  __ush_for_list_index_11=$((__ush_for_list_index_11 + 1))
  __ush_at 50 'for (config, invariant) in [("Version", "CurrentRevision"), ("Cancelled", "LiveRequest"), ("Partial", "NoPartialOutput")] {'
  __ush_binding_50_config="${__ush_for_50__0}"
  # shellcheck disable=SC2034
  config="${__ush_binding_50_config}"
  __ush_at 50 'for (config, invariant) in [("Version", "CurrentRevision"), ("Cancelled", "LiveRequest"), ("Partial", "NoPartialOutput")] {'
  __ush_binding_50_invariant="${__ush_for_50__1}"
  # shellcheck disable=SC2034
  invariant="${__ush_binding_50_invariant}"
  __ush_at 51 'require_violation $tlc_jar $"$temp/request-$config" $"Request$config.cfg" Request.tla $invariant $"$temp/request-$config.log"'
  ( __ush_capture_return='0' __ush_return_path=''; ush_fn_require_violation "${__ush_binding_30_tlc_jar}" "${__ush_binding_27_temp}"'/request-'"${__ush_binding_50_config}" 'Request'"${__ush_binding_50_config}"'.cfg' 'Request.tla' "${__ush_binding_50_invariant}" "${__ush_binding_27_temp}"'/request-'"${__ush_binding_50_config}"'.log' )
done
