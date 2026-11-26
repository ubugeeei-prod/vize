#!/bin/sh
# Generated from '../verify-current-source-cli-configuration-entries-and-independent-witnesse.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../verify-current-source-cli-configuration-entries-and-independent-witnesse.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../verify-current-source-cli-configuration-entries-and-independent-witnesse.ush' --checked-in -o 'verify-current-source-cli-configuration-entries-and-independent-witnesse.sh'
set -eu

__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'verify-current-source-cli-configuration-entries-and-independent-witnesse.ush'
__ush_origin='../verify-current-source-cli-configuration-entries-and-independent-witnesse.ush'

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
  -h|--help) printf '%s' 'verify-current-source-cli-configuration-entries-and-independent-witnesse.ush

Usage:
  verify-current-source-cli-configuration-entries-and-independent-witnesse.ush [--help]
  verify-current-source-cli-configuration-entries-and-independent-witnesse.ush [--man [ITEM]]
  verify-current-source-cli-configuration-entries-and-independent-witnesse.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
verify-current-source-cli-configuration-entries-and-independent-witnesse.ush

SYNOPSIS
verify-current-source-cli-configuration-entries-and-independent-witnesse.ush [--help]
verify-current-source-cli-configuration-entries-and-independent-witnesse.ush [--man [ITEM]]
verify-current-source-cli-configuration-entries-and-independent-witnesse.ush [--complete [PREFIX]]
'; exit 0 ;;
  complete|--complete)
    for item in --complete --help --man -h complete man; do
      case $item in "${2:-}"*) printf '%s\n' "$item" ;; esac
    done
    exit 0 ;;
esac

__ush_at 4 'let evidence = $"target/n8n-cli-config/$env::GITHUB_RUN_ID-$env::GITHUB_RUN_ATTEMPT"'
__ush_env_0=${GITHUB_RUN_ID-}
if [ -z "${__ush_env_0}" ]; then __ush_report 1 'environment variable GITHUB_RUN_ID is not set or is empty'; exit 1; fi
__ush_env_1=${GITHUB_RUN_ATTEMPT-}
if [ -z "${__ush_env_1}" ]; then __ush_report 1 'environment variable GITHUB_RUN_ATTEMPT is not set or is empty'; exit 1; fi
__ush_binding_4_evidence='target/n8n-cli-config/'"${__ush_env_0}"'-'"${__ush_env_1}"
# shellcheck disable=SC2034
evidence="${__ush_binding_4_evidence}"
__ush_at 6 'vp node --test tests/tooling/n8n-cli-config-contract.test.mjs tests/tooling/n8n-cli-config-failure-custody.test.mjs'
command vp 'node' '--test' 'tests/tooling/n8n-cli-config-contract.test.mjs' 'tests/tooling/n8n-cli-config-failure-custody.test.mjs'
__ush_at 7 'vp node tests/tooling/support/n8n-cli-config-acceptance.mjs $evidence'
command vp 'node' 'tests/tooling/support/n8n-cli-config-acceptance.mjs' "${__ush_binding_4_evidence}"
__ush_at 8 'vp node tests/tooling/support/n8n-cli-config-semantic-acceptance.mjs $"$evidence/semantic-routing"'
command vp 'node' 'tests/tooling/support/n8n-cli-config-semantic-acceptance.mjs' "${__ush_binding_4_evidence}"'/semantic-routing'
