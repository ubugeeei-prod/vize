#!/bin/sh
# Generated from '../build-the-actual-source-compiler-and-both-public-package-entrypoints.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../build-the-actual-source-compiler-and-both-public-package-entrypoints.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../build-the-actual-source-compiler-and-both-public-package-entrypoints.ush' --checked-in -o 'build-the-actual-source-compiler-and-both-public-package-entrypoints.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'build-the-actual-source-compiler-and-both-public-package-entrypoints.ush'
__ush_origin='../build-the-actual-source-compiler-and-both-public-package-entrypoints.ush'

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
  -h|--help) printf '%s' 'build-the-actual-source-compiler-and-both-public-package-entrypoints.ush

Usage:
  build-the-actual-source-compiler-and-both-public-package-entrypoints.ush [--help]
  build-the-actual-source-compiler-and-both-public-package-entrypoints.ush [--man [ITEM]]
  build-the-actual-source-compiler-and-both-public-package-entrypoints.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
build-the-actual-source-compiler-and-both-public-package-entrypoints.ush

SYNOPSIS
build-the-actual-source-compiler-and-both-public-package-entrypoints.ush [--help]
build-the-actual-source-compiler-and-both-public-package-entrypoints.ush [--man [ITEM]]
build-the-actual-source-compiler-and-both-public-package-entrypoints.ush [--complete [PREFIX]]
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

ush_fn_std__run_words() {
  # Run a program whose arguments are tagged words: `s value` is one argument,
  # `l packed` gives every element of a List an argument of its own.
  __ush_program=$1
  shift
  __ush_argv__len=0
  while [ "$#" -gt 0 ]; do
    case $1 in
      s)
        eval "__ush_argv__${__ush_argv__len}=\$2"
        __ush_argv__len=$((__ush_argv__len + 1))
        ;;
      l)
        __ush_words__len=0
        ush_fn_std__unpack __ush_words "$2" || return "$?"
        __ush_index=0
        while [ "$__ush_index" -lt "$__ush_words__len" ]; do
          eval "__ush_argv__${__ush_argv__len}=\${__ush_words__${__ush_index}}"
          __ush_argv__len=$((__ush_argv__len + 1))
          __ush_index=$((__ush_index + 1))
        done
        ;;
    esac
    shift 2
  done
  set --
  __ush_index=0
  while [ "$__ush_index" -lt "$__ush_argv__len" ]; do
    eval "__ush_arg=\${__ush_argv__${__ush_index}}"
    set -- "$@" "$__ush_arg"
    __ush_index=$((__ush_index + 1))
  done
  command "$__ush_program" "$@"
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

ush_fn_std__path__extension() {
  __ush_base=${1##*/}
  case $__ush_base in .|..|.*) __ush_base=${__ush_base#?} ;; esac
  case $__ush_base in *.*) printf '%s' "${__ush_base##*.}" ;; esac
}

ush_fn_std__process__chdir() {
  __ush_std_fs_path "$1" || return "$?"
  CDPATH='' cd "$__ush_path" || return "$?"
}

__ush_at 7 'let evidence = $"$env::RUNNER_TEMP/babel-plugin"'
__ush_env_0=${RUNNER_TEMP-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable RUNNER_TEMP is not set or is empty'; exit 1; fi
__ush_binding_7_evidence="${__ush_env_0}"'/babel-plugin'
# shellcheck disable=SC2034
evidence="${__ush_binding_7_evidence}"
__ush_at 8 'let packed = $"$evidence/packed"'
__ush_binding_8_packed="${__ush_binding_7_evidence}"'/packed'
# shellcheck disable=SC2034
packed="${__ush_binding_8_packed}"
__ush_at 10 'if (git rev-parse HEAD) != $env::SOURCE_SHA {'
if {
__ush_value_1="$(__ush_return_path='' __ush_capture_return='1' command git 'rev-parse' 'HEAD')" || exit "$?"
__ush_env_2=${SOURCE_SHA-}
if [ -z "${__ush_env_2}" ]; then __ush_report 1 'environment variable SOURCE_SHA is not set or is empty'; exit 1; fi
[ "$(if [ "${__ush_value_1}" != "${__ush_env_2}" ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 11 'fail 1 $"The checkout is not the source commit $env::SOURCE_SHA"'
  __ush_env_3=${SOURCE_SHA-}
  if [ -z "${__ush_env_3}" ]; then __ush_report 1 'environment variable SOURCE_SHA is not set or is empty'; exit 1; fi
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'The checkout is not the source commit '"${__ush_env_3}"
  exit "$__ush_status"
fi
__ush_at 13 'mkdir_p $packed'
( __ush_capture_return='0' __ush_return_path=''; ush_fn_std__fs__mkdir_p "${__ush_binding_8_packed}" )
__ush_at 14 'git rev-parse HEAD "HEAD^{tree}" | to_file $evidence/source.txt'
__ush_value_4="${__ush_binding_7_evidence}"
__ush_std_fs_path "${__ush_value_4}"'/source.txt' || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
__ush_path_5=$__ush_path
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (command git 'rev-parse' 'HEAD' 'HEAD^{tree}') 7>&- >"$__ush_path_5"; printf ' 1:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: git rev-parse HEAD "HEAD^{tree}" | to_file $evidence/sour...' || exit "$?"
__ush_at 15 'git log -50 "--format=%H %T %P %s" | to_file $evidence/ordered-history.txt'
__ush_value_6="${__ush_binding_7_evidence}"
__ush_std_fs_path "${__ush_value_6}"'/ordered-history.txt' || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
__ush_path_7=$__ush_path
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (command git 'log' -50 '--format=%H %T %P %s') 7>&- >"$__ush_path_7"; printf ' 1:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: git log -50 "--format=%H %T %P %s" | to_file $evidence/or...' || exit "$?"
__ush_at 16 'node --version | to_file $evidence/node-version.txt'
__ush_value_8="${__ush_binding_7_evidence}"
__ush_std_fs_path "${__ush_value_8}"'/node-version.txt' || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
__ush_path_9=$__ush_path
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (command node '--version') 7>&- >"$__ush_path_9"; printf ' 1:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: node --version | to_file $evidence/node-version.txt' || exit "$?"
__ush_at 17 'vp run --workspace-root build:native:test'
command vp 'run' '--workspace-root' 'build:native:test'
__ush_at 18 'export GITHUB_SHA = $env::SOURCE_SHA {'
__ush_env_10=${SOURCE_SHA-}
if [ -z "${__ush_env_10}" ]; then __ush_report 1 'environment variable SOURCE_SHA is not set or is empty'; exit 1; fi
__ush_had_GITHUB_SHA_11=${GITHUB_SHA+x} __ush_saved_GITHUB_SHA_11=${GITHUB_SHA-}
export GITHUB_SHA="${__ush_env_10}"
  __ush_at 19 'node npm/native/scripts/native-build-receipt.mjs'
  command node 'npm/native/scripts/native-build-receipt.mjs'
if [ -n "$__ush_had_GITHUB_SHA_11" ]; then export GITHUB_SHA="$__ush_saved_GITHUB_SHA_11"; else unset GITHUB_SHA; fi
__ush_at 21 'vp run --no-cache --filter ./npm/builder/babel build'
command vp 'run' '--no-cache' '--filter' './npm/builder/babel' 'build'
__ush_at 22 'cd "npm/builder/babel" {'
__ush_cwd_12=$PWD
ush_fn_std__process__chdir 'npm/builder/babel' || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
  __ush_at 23 'npm pack --ignore-scripts --pack-destination $packed --json | to_file $evidence/pack.json'
  __ush_value_13="${__ush_binding_7_evidence}"
  __ush_std_fs_path "${__ush_value_13}"'/pack.json' || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
  __ush_path_14=$__ush_path
  { __ush_pipe=$(
    exec 7>&1 >&8 8>&-
    set +e
    { (command npm 'pack' '--ignore-scripts' '--pack-destination' "${__ush_binding_8_packed}" '--json') 7>&- >"$__ush_path_14"; printf ' 1:%s' "$?" >&7; }
  ); } 8>&1
  __ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: npm pack --ignore-scripts --pack-destination $packed --js...' || exit "$?"
cd "$__ush_cwd_12" || { __ush_status=$?; __ush_report "$__ush_status" 'cannot return to the directory the cd block started in'; exit "$__ush_status"; }
__ush_at 25 'let tarballs = [file for file in read_dir $packed if (extension $file) == "tgz"]'
__ush_comprehension_count_15=0
__ush_packed_17="$(__ush_return_path='' __ush_capture_return=1 ush_fn_std__fs__read_dir "${__ush_binding_8_packed}")" || exit "$?"
__ush_match_16__len=0
ush_fn_std__unpack __ush_match_16 "${__ush_packed_17}"
__ush_comprehension_index_18=0
while [ "${__ush_comprehension_index_18}" -lt "${__ush_match_16__len}" ]; do
eval "__ush_comprehension_item_19=\"\${__ush_match_16__${__ush_comprehension_index_18}}\""
__ush_value_20="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__extension "${__ush_comprehension_item_19}")" || exit "$?"
if [ "$(if [ "${__ush_value_20}" = 'tgz' ]; then printf '%s' true; else printf '%s' false; fi)" = true ]; then
# shellcheck disable=SC2034
__ush_comprehension_value_21="${__ush_comprehension_item_19}"
eval "__ush_binding_25_tarballs__${__ush_comprehension_count_15}=\$__ush_comprehension_value_21"
__ush_comprehension_count_15=$((__ush_comprehension_count_15 + 1))
fi
__ush_comprehension_index_18=$((__ush_comprehension_index_18 + 1))
done
__ush_binding_25_tarballs__len="${__ush_comprehension_count_15}"
__ush_at 26 'if tarballs.is_empty() {'
if {
if [ "${__ush_binding_25_tarballs__len}" -eq 0 ]; then __ush_value_22=true; else __ush_value_22=false; fi
[ "${__ush_value_22}" = 'true' ]
}; then
  __ush_at 27 'fail 1 $"npm pack wrote no tarball to $packed"'
  __ush_status=1
  if [ "$__ush_status" -lt 1 ] || [ "$__ush_status" -gt 255 ]; then __ush_status=1; fi
  __ush_report "$__ush_status" 'npm pack wrote no tarball to '"${__ush_binding_8_packed}"
  exit "$__ush_status"
fi
__ush_at 29 'sha256sum $tarballs | to_file $evidence/packed.sha256'
__ush_match_23__len="${__ush_binding_25_tarballs__len}"
__ush_copy_index_24=0
while [ "$__ush_copy_index_24" -lt "${__ush_binding_25_tarballs__len}" ]; do
  eval "__ush_copy_value_25=\"\${__ush_binding_25_tarballs__$__ush_copy_index_24}\""
  eval "__ush_match_23__${__ush_copy_index_24}=\$__ush_copy_value_25"
  __ush_copy_index_24=$((__ush_copy_index_24 + 1))
done
__ush_value_27="${__ush_binding_7_evidence}"
__ush_std_fs_path "${__ush_value_27}"'/packed.sha256' || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
__ush_path_28=$__ush_path
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (ush_fn_std__run_words 'sha256sum' l "$(ush_fn_std__pack __ush_match_23)") 7>&- >"$__ush_path_28"; printf ' 1:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: sha256sum $tarballs | to_file $evidence/packed.sha256' || exit "$?"
__ush_at 30 'cp npm/native/.artifacts/native/build-token-receipt.json $evidence/'
__ush_value_29="${__ush_binding_7_evidence}"
command cp 'npm/native/.artifacts/native/build-token-receipt.json' "${__ush_value_29}"'/'
