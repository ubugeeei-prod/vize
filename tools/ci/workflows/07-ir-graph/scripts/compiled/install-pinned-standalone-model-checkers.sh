#!/bin/sh
# Generated from '../install-pinned-standalone-model-checkers.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../install-pinned-standalone-model-checkers.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../install-pinned-standalone-model-checkers.ush' --checked-in -o 'install-pinned-standalone-model-checkers.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'install-pinned-standalone-model-checkers.ush'
__ush_origin='../install-pinned-standalone-model-checkers.ush'

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
  -h|--help) printf '%s' 'install-pinned-standalone-model-checkers.ush

Usage:
  install-pinned-standalone-model-checkers.ush [--help]
  install-pinned-standalone-model-checkers.ush [--man [ITEM]]
  install-pinned-standalone-model-checkers.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
install-pinned-standalone-model-checkers.ush

SYNOPSIS
install-pinned-standalone-model-checkers.ush [--help]
install-pinned-standalone-model-checkers.ush [--man [ITEM]]
install-pinned-standalone-model-checkers.ush [--complete [PREFIX]]
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

__ush_at 4 'let checksums = """'
__ush_binding_4_checksums='6037cbeee0e8423c1c468447ed10f5fcf2f2743a2ffc39cb1c81f2905c0fdb9d  alloy.jar'"${__ush_lf}"'936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88  tlc.jar'"${__ush_lf}"'890afd185370f85666025b883914ab4f4b339136f8c96167b69cfb62aecaf235  lean.tar.zst'
# shellcheck disable=SC2034
checksums="${__ush_binding_4_checksums}"
__ush_at 10 'curl -fsSL --retry 3 -o alloy.jar https://repo.maven.apache.org/maven2/org/alloytools/org.alloytools.alloy.dist/6.2.0/org.alloytools.alloy.dist-6.2.0.jar'
command curl '-fsSL' '--retry' 3 '-o' 'alloy.jar' 'https://repo.maven.apache.org/maven2/org/alloytools/org.alloytools.alloy.dist/6.2.0/org.alloytools.alloy.dist-6.2.0.jar'
__ush_at 11 'curl -fsSL --retry 3 -o tlc.jar https://github.com/tlaplus/tlaplus/releases/download/v1.7.4/tla2tools.jar'
command curl '-fsSL' '--retry' 3 '-o' 'tlc.jar' 'https://github.com/tlaplus/tlaplus/releases/download/v1.7.4/tla2tools.jar'
__ush_at 12 'curl -fsSL --retry 3 -o lean.tar.zst https://github.com/leanprover/lean4/releases/download/v4.33.1/lean-4.33.1-linux.tar.zst'
command curl '-fsSL' '--retry' 3 '-o' 'lean.tar.zst' 'https://github.com/leanprover/lean4/releases/download/v4.33.1/lean-4.33.1-linux.tar.zst'
__ush_at 13 '$"$checksums\n" | sha256sum -c'
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (printf '%s' "${__ush_binding_4_checksums}""${__ush_lf}") 7>&-; printf ' 1:%s' "$?" >&7; } |
  { (command sha256sum '-c') 7>&-; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: $"$checksums\n"' 'pipeline stage 2 failed: sha256sum -c' || exit "$?"
__ush_at 14 'tar --zstd -xf lean.tar.zst'
command tar '--zstd' '-xf' 'lean.tar.zst'
