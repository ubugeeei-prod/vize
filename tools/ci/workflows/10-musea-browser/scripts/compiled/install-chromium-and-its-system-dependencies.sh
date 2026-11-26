#!/bin/sh
# Generated from '../install-chromium-and-its-system-dependencies.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../install-chromium-and-its-system-dependencies.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../install-chromium-and-its-system-dependencies.ush' --checked-in -o 'install-chromium-and-its-system-dependencies.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'install-chromium-and-its-system-dependencies.ush'
__ush_origin='../install-chromium-and-its-system-dependencies.ush'

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
  -h|--help) printf '%s' 'install-chromium-and-its-system-dependencies.ush

Usage:
  install-chromium-and-its-system-dependencies.ush [--help]
  install-chromium-and-its-system-dependencies.ush [--man [ITEM]]
  install-chromium-and-its-system-dependencies.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
install-chromium-and-its-system-dependencies.ush

SYNOPSIS
install-chromium-and-its-system-dependencies.ush [--help]
install-chromium-and-its-system-dependencies.ush [--man [ITEM]]
install-chromium-and-its-system-dependencies.ush [--complete [PREFIX]]
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

__ush_at 6 'let apt_config = """'
__ush_binding_6_apt_config='Acquire::Languages "none";'"${__ush_lf}"'Acquire::IndexTargets::deb::DEP-11::DefaultEnabled "false";'"${__ush_lf}"'Acquire::IndexTargets::deb::CNF::DefaultEnabled "false";'
# shellcheck disable=SC2034
apt_config="${__ush_binding_6_apt_config}"
__ush_at 12 '$"$apt_config\n" | sudo tee /etc/apt/apt.conf.d/81-vize-musea-package-indexes | to_file "/dev/null"'
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (printf '%s' "${__ush_binding_6_apt_config}""${__ush_lf}") 7>&-; printf ' 1:%s' "$?" >&7; } |
  { (command sudo 'tee' '/etc/apt/apt.conf.d/81-vize-musea-package-indexes') 7>&- >'/dev/null'; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: $"$apt_config\n"' 'pipeline stage 2 failed: sudo tee /etc/apt/apt.conf.d/81-vize-musea-package-indexe...' || exit "$?"
__ush_at 13 'apt_retry vp exec --filter ./npm/builder/vite-musea -- playwright install --with-deps chromium'
command vize-ci-apt-retry 'vp' 'exec' '--filter' './npm/builder/vite-musea' '--' 'playwright' 'install' '--with-deps' 'chromium'
