#!/bin/sh
# Generated from '../file-canonical-divergence-issue.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../file-canonical-divergence-issue.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../file-canonical-divergence-issue.ush' --checked-in -o 'file-canonical-divergence-issue.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'file-canonical-divergence-issue.ush'
__ush_origin='../file-canonical-divergence-issue.ush'

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
  -h|--help) printf '%s' 'file-canonical-divergence-issue.ush

Usage:
  file-canonical-divergence-issue.ush [--help]
  file-canonical-divergence-issue.ush [--man [ITEM]]
  file-canonical-divergence-issue.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
file-canonical-divergence-issue.ush

SYNOPSIS
file-canonical-divergence-issue.ush [--help]
file-canonical-divergence-issue.ush [--man [ITEM]]
file-canonical-divergence-issue.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

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

ush_fn_std__test() {
  __ush_test=0
  # Preserve test's status 2 (invalid arguments), distinct from false/status 1.
  # shellcheck disable=SC2319
  test "$@" || __ush_test=$?
  if [ "$__ush_test" -eq 0 ]; then printf true; else
    if [ "$__ush_test" -eq 1 ]; then printf false; else return "$__ush_test"; fi
  fi
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

__ush_at 5 'let observer = $env::OBSERVER'
__ush_env_0=${OBSERVER-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable OBSERVER is not set or is empty'; exit 1; fi
__ush_binding_5_observer="${__ush_env_0}"
# shellcheck disable=SC2034
observer="${__ush_binding_5_observer}"
__ush_at 6 'let promotion = $"real-project-$observer-promotion"'
__ush_binding_6_promotion='real-project-'"${__ush_binding_5_observer}"'-promotion'
# shellcheck disable=SC2034
promotion="${__ush_binding_6_promotion}"
__ush_at 8 'let mut evidence = [$"real-project-$observer-corpus", $"real-project-$observer-reduction"]'
__ush_binding_8_evidence__len='2'
# shellcheck disable=SC2034
__ush_binding_8_evidence__0='real-project-'"${__ush_binding_5_observer}"'-corpus'
# shellcheck disable=SC2034
__ush_binding_8_evidence__1='real-project-'"${__ush_binding_5_observer}"'-reduction'
__ush_at 10 'if test -s $"$promotion/promotion.json" {'
if {
__ush_value_1="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__test '-s' "${__ush_binding_6_promotion}"'/promotion.json')" || exit "$?"
[ "${__ush_value_1}" = 'true' ]
}; then
  __ush_at 11 'evidence = evidence + [promotion]'
  __ush_match_2__len='1'
  # shellcheck disable=SC2034
  __ush_match_2__0="${__ush_binding_6_promotion}"
  __ush_count_3=${__ush_binding_8_evidence__len}
  __ush_index_4=0
  while [ "${__ush_index_4}" -lt "${__ush_match_2__len}" ]; do
  eval "__ush_binding_8_evidence__${__ush_count_3}=\"\${__ush_match_2__${__ush_index_4}}\""
    __ush_index_4=$((__ush_index_4 + 1))
    __ush_count_3=$((__ush_count_3 + 1))
  done
  __ush_binding_8_evidence__len=${__ush_count_3}
fi
__ush_at 13 'exec node (["tools/support/compat/github/canonical-corpus-divergence-issue.mjs", "publish", observer] + evidence)'
__ush_match_6__len='3'
# shellcheck disable=SC2034
__ush_match_6__0='tools/support/compat/github/canonical-corpus-divergence-issue.mjs'
# shellcheck disable=SC2034
__ush_match_6__1='publish'
# shellcheck disable=SC2034
__ush_match_6__2="${__ush_binding_5_observer}"
__ush_count_7=0
__ush_index_8=0
while [ "${__ush_index_8}" -lt "${__ush_match_6__len}" ]; do
eval "__ush_match_5__${__ush_count_7}=\"\${__ush_match_6__${__ush_index_8}}\""
  __ush_index_8=$((__ush_index_8 + 1))
  __ush_count_7=$((__ush_count_7 + 1))
done
__ush_index_9=0
while [ "${__ush_index_9}" -lt "${__ush_binding_8_evidence__len}" ]; do
eval "__ush_match_5__${__ush_count_7}=\"\${__ush_binding_8_evidence__${__ush_index_9}}\""
  __ush_index_9=$((__ush_index_9 + 1))
  __ush_count_7=$((__ush_count_7 + 1))
done
__ush_match_5__len=${__ush_count_7}
( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__process__exec 'node' "$(ush_fn_std__pack __ush_match_5)" )
