#!/bin/sh
# Generated from '../download-chromium-dependencies-while-building-the-current-cli.ush' by ush.
# DO NOT EDIT MANUALLY.
# Source: ../download-chromium-dependencies-while-building-the-current-cli.ush
# Source map: embedded __ush_at markers; ush explain SOURCE LINE
# Regenerate with: ush compile '../download-chromium-dependencies-while-building-the-current-cli.ush' --checked-in -o 'download-chromium-dependencies-while-building-the-current-cli.sh'
set -eu

__ush_lf='
'
__ush_entrypoint=$0
case $__ush_entrypoint in
  */*) ;;
  *) if [ -f "$__ush_entrypoint" ]; then __ush_entrypoint=./$__ush_entrypoint; else __ush_entrypoint=$(command -v "$__ush_entrypoint"); fi ;;
esac
__ush_source_dir=$(CDPATH='' cd -P "$(dirname "$__ush_entrypoint")"/'..' && pwd -P)
__ush_source_path=$__ush_source_dir/'download-chromium-dependencies-while-building-the-current-cli.ush'
__ush_origin='../download-chromium-dependencies-while-building-the-current-cli.ush'

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
  -h|--help) printf '%s' 'download-chromium-dependencies-while-building-the-current-cli.ush

Usage:
  download-chromium-dependencies-while-building-the-current-cli.ush [--help]
  download-chromium-dependencies-while-building-the-current-cli.ush [--man [ITEM]]
  download-chromium-dependencies-while-building-the-current-cli.ush [--complete [PREFIX]]
'; exit 0 ;;
  man|--man)
    if [ -n "${2:-}" ]; then printf 'No documented item named: %s\n' "$2" >&2; exit 1; fi
    printf '%s' 'NAME
download-chromium-dependencies-while-building-the-current-cli.ush

SYNOPSIS
download-chromium-dependencies-while-building-the-current-cli.ush [--help]
download-chromium-dependencies-while-building-the-current-cli.ush [--man [ITEM]]
download-chromium-dependencies-while-building-the-current-cli.ush [--complete [PREFIX]]
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

# Run the deferred blocks of the current scope, last registered first. A
# block that fails has reported its own line; the rest still run, and the
# first failure becomes the exit status if the scope was succeeding.
__ush_run_defers() {
  while [ -n "${__ush_defers:-}" ]; do
    __ush_defer=${__ush_defers%% *}
    __ush_defers=${__ush_defers#* }
    "$__ush_defer" || {
      __ush_status=$?
      [ "$__ush_exit" -ne 0 ] || __ush_exit=$__ush_status
    }
  done
}
# A function that defers runs in a process of its own; its exit, including
# one caused by a handled signal, runs its blocks.
__ush_defer_scope() {
  __ush_signal=''
  __ush_defers=''
  trap '__ush_defer_exit "${__ush_signal:-$?}"' 0
  trap '__ush_signal=129; exit 129' HUP
  trap '__ush_signal=130; exit 130' INT
  trap '__ush_signal=143; exit 143' TERM
}
__ush_defer_exit() {
  __ush_exit=$1
  __ush_run_defers
  exit "$__ush_exit"
}

# Send signal $2 to task $1, whose status file is $3, and every process it
# started. A task this scope has already reaped is left alone: its ID may
# belong to another process by now. A killed task's status does not fail
# the scope when it is joined; awaiting it still reports the signal.
__ush_task_kill() {
  case $2 in
    HUP|INT|QUIT|KILL|USR1|USR2|ALRM|TERM|STOP|CONT) ;;
    *) __ush_report 2 "unknown signal: $2"; return 2 ;;
  esac
  case " $__ush_jobs " in *" $1 "*) ;; *) return 0 ;; esac
  # A task that has not started yet still holds the traps of the shell it
  # was forked from; wait until it has dropped them.
  while [ ! -e "$3.started" ] && kill -0 "$1" 2>/dev/null; do
    sleep 0.01 2>/dev/null || sleep 1
  done
  __ush_killed="${__ush_killed:-} $1"
  # The task itself is this shell's child and is not stopped: some shells
  # would report the stop to a later wait as the task's status.
  __ush_kill_children=$(__ush_child_pids "$1")
  kill -s "$2" "$1" 2>/dev/null || :
  for __ush_kill_child in $__ush_kill_children; do
    __ush_kill_tree "$__ush_kill_child" "$2"
  done
}
# Each descendant is stopped before its children are listed, so none can
# start another one unseen, signalled while stopped, and continued once its
# children are signalled, so a stopped process receives the signal.
__ush_kill_tree() {
  case $2 in STOP|CONT) ;; *) kill -s STOP "$1" 2>/dev/null || return 0 ;; esac
  __ush_kill_children=$(__ush_child_pids "$1")
  kill -s "$2" "$1" 2>/dev/null || :
  for __ush_kill_child in $__ush_kill_children; do
    __ush_kill_tree "$__ush_kill_child" "$2"
  done
  [ "$2" = STOP ] || kill -s CONT "$1" 2>/dev/null || :
}
__ush_child_pids() {
  { ps -A -o pid= -o ppid= 2>/dev/null || ps -o pid= -o ppid=; } |
    awk -v parent="$1" '$2 == parent { print $1 }'
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

ush_fn_std__process__exit() {
  case $1 in ''|*[!0-9]*) return 1 ;; esac
  [ "$1" -le 255 ] || return 1
  exit "$1"
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

# raises: unknown
ush_fn_download_chromium_dependencies() {
  __ush_at 7 'cd "tests" {'
  __ush_cwd_0=$PWD
  ush_fn_std__process__chdir 'tests' || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
    __ush_at 8 'apt_retry vp exec playwright install-deps chromium'
    command vize-ci-apt-retry 'vp' 'exec' 'playwright' 'install-deps' 'chromium' || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
  cd "$__ush_cwd_0" || { __ush_status=$?; __ush_report "$__ush_status" 'cannot return to the directory the cd block started in'; return "$__ush_status"; }
}

__ush_at 12 'let inventory_hook = from_source "compiled/report-apt-archive-inventory.sh"'
__ush_value_1="$(__ush_return_path='' __ush_capture_return='1' ush_fn_std__path__from_source 'compiled/report-apt-archive-inventory.sh')" || exit "$?"
__ush_binding_12_inventory_hook="${__ush_value_1}"
# shellcheck disable=SC2034
inventory_hook="${__ush_binding_12_inventory_hook}"
__ush_at 16 'let archive_dir = sudo mktemp -d /var/cache/apt/vize-nuxt-archives-XXXXXX'
__ush_value_2="$(__ush_return_path='' __ush_capture_return='1' command sudo 'mktemp' '-d' '/var/cache/apt/vize-nuxt-archives-XXXXXX')" || exit "$?"
__ush_binding_16_archive_dir="${__ush_value_2}"
# shellcheck disable=SC2034
archive_dir="${__ush_binding_16_archive_dir}"
__ush_at 17 'defer sudo rm -rf -- $archive_dir'
__ush_defer_3() {
  __ush_at 17 'defer sudo rm -rf -- $archive_dir'
  command sudo 'rm' '-rf' '--' "${__ush_binding_16_archive_dir}" || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
}
__ush_defers="__ush_defer_3 ${__ush_defers:-}"
__ush_at 19 'sudo chmod "0755" $archive_dir'
command sudo 'chmod' '0755' "${__ush_binding_16_archive_dir}"
__ush_at 20 'sudo install -m "0755" $inventory_hook $archive_dir/report-inventory'
__ush_value_4="${__ush_binding_16_archive_dir}"
command sudo 'install' '-m' '0755' "${__ush_binding_12_inventory_hook}" "${__ush_value_4}"'/report-inventory'
__ush_at 21 'let cache_config = sudo mktemp /etc/apt/apt.conf.d/99-vize-nuxt-archives-XXXXXX'
__ush_value_5="$(__ush_return_path='' __ush_capture_return='1' command sudo 'mktemp' '/etc/apt/apt.conf.d/99-vize-nuxt-archives-XXXXXX')" || exit "$?"
__ush_binding_21_cache_config="${__ush_value_5}"
# shellcheck disable=SC2034
cache_config="${__ush_binding_21_cache_config}"
__ush_at 22 'defer sudo rm -f -- $cache_config'
__ush_defer_6() {
  __ush_at 22 'defer sudo rm -f -- $cache_config'
  command sudo 'rm' '-f' '--' "${__ush_binding_21_cache_config}" || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
}
__ush_defers="__ush_defer_6 ${__ush_defers:-}"
__ush_at 24 'let download_config = sudo mktemp /etc/apt/apt.conf.d/98-vize-nuxt-download-only-XXXXXX'
__ush_value_7="$(__ush_return_path='' __ush_capture_return='1' command sudo 'mktemp' '/etc/apt/apt.conf.d/98-vize-nuxt-download-only-XXXXXX')" || exit "$?"
__ush_binding_24_download_config="${__ush_value_7}"
# shellcheck disable=SC2034
download_config="${__ush_binding_24_download_config}"
__ush_at 25 'defer sudo rm -f -- $download_config'
__ush_defer_8() {
  __ush_at 25 'defer sudo rm -f -- $download_config'
  command sudo 'rm' '-f' '--' "${__ush_binding_24_download_config}" || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
}
__ush_defers="__ush_defer_8 ${__ush_defers:-}"
__ush_at 26 'let cache_settings = $"""'
__ush_binding_26_cache_settings='Dir::Cache::archives "'"${__ush_binding_16_archive_dir}"'";'"${__ush_lf}"'APT::Update::Post-Invoke { "'"'"''"${__ush_binding_16_archive_dir}"'/report-inventory'"'"'"; };'
# shellcheck disable=SC2034
cache_settings="${__ush_binding_26_cache_settings}"
__ush_at 30 '$"$cache_settings\n" | sudo tee $cache_config | to_file "/dev/null"'
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (printf '%s' "${__ush_binding_26_cache_settings}""${__ush_lf}") 7>&-; printf ' 1:%s' "$?" >&7; } |
  { (command sudo 'tee' "${__ush_binding_21_cache_config}") 7>&- >'/dev/null'; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: $"$cache_settings\n"' 'pipeline stage 2 failed: sudo tee $cache_config | to_file "/dev/null"' || exit "$?"
__ush_at 31 '"APT::Get::Download-Only \"true\";\n" | sudo tee $download_config | to_file "/dev/null"'
{ __ush_pipe=$(
  exec 7>&1 >&8 8>&-
  set +e
  { (printf '%s' 'APT::Get::Download-Only "true";'"${__ush_lf}"'') 7>&-; printf ' 1:%s' "$?" >&7; } |
  { (command sudo 'tee' "${__ush_binding_24_download_config}") 7>&- >'/dev/null'; printf ' 2:%s' "$?" >&7; }
); } 8>&1
__ush_pipe_check "$__ush_pipe" 'pipeline stage 1 failed: "APT::Get::Download-Only \"true\";\n"' 'pipeline stage 2 failed: sudo tee $download_config | to_file "/dev/null"' || exit "$?"
__ush_at 33 'let cli = spawn cargo build --profile ci -p vize'
__ush_task_init
__ush_at 33 'let cli = spawn cargo build --profile ci -p vize'
ush_fn___ush_task_block_9() {
  __ush_at 33 'let cli = spawn cargo build --profile ci -p vize'
  command cargo 'build' '--profile' 'ci' '-p' 'vize' || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
}
__ush_task_slot
__ush_task_10__result=$__ush_slot/result
: > "${__ush_task_10__result}"
: > "${__ush_task_10__result}.status"
( __ush_task_scope "${__ush_task_10__result}.status"; __ush_return_path="${__ush_task_10__result}"; ush_fn___ush_task_block_9 ) &
__ush_task_10__pid=$!
__ush_task_10__awaited=0
__ush_jobs="${__ush_jobs} $!"
__ush_binding_33_cli__pid="${__ush_task_10__pid}"
__ush_binding_33_cli__result="${__ush_task_10__result}"
__ush_binding_33_cli__awaited="${__ush_task_10__awaited}"
__ush_at 34 'defer cli.kill()'
__ush_defer_11() {
  __ush_at 34 'defer cli.kill()'
  __ush_task_kill "${__ush_binding_33_cli__pid}" TERM "${__ush_binding_33_cli__result}.status" || { __ush_status=$?; __ush_report "$__ush_status"; return "$__ush_status"; }
}
__ush_defers="__ush_defer_11 ${__ush_defers:-}"
__ush_at 35 'let download_status = status_of download_chromium_dependencies'
if ( __ush_capture_return='0' __ush_return_path=''; ush_fn_download_chromium_dependencies ); then __ush_status_12=0; else __ush_status_12=$?; fi
__ush_binding_35_download_status="${__ush_status_12}"
# shellcheck disable=SC2034
download_status="${__ush_binding_35_download_status}"
__ush_at 36 'let cli_status = status_of cli.await'
if { [ "${__ush_binding_33_cli__awaited}" = '1' ] || { __ush_wait_job "${__ush_binding_33_cli__pid}" "${__ush_binding_33_cli__result}.status" && __ush_binding_33_cli__awaited='1'; }; }; then __ush_status_13=0; else __ush_status_13=$?; fi
__ush_binding_36_cli_status="${__ush_status_13}"
# shellcheck disable=SC2034
cli_status="${__ush_binding_36_cli_status}"
__ush_at 37 'sudo rm -- $download_config'
command sudo 'rm' '--' "${__ush_binding_24_download_config}"
__ush_at 38 'echo $"Nuxt3 setup statuses: download=$download_status CLI=$cli_status"'
printf '%s\n' 'Nuxt3 setup statuses: download='"${__ush_binding_35_download_status}"' CLI='"${__ush_binding_36_cli_status}"
__ush_at 39 'if download_status != 0 {'
if {
[ "$(if [ "${__ush_binding_35_download_status}" -ne 0 ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 40 'std::process::exit $download_status'
  ush_fn_std__process__exit "${__ush_binding_35_download_status}"
fi
__ush_at 42 'if cli_status != 0 {'
if {
[ "$(if [ "${__ush_binding_36_cli_status}" -ne 0 ]; then printf '%s' true; else printf '%s' false; fi)" = 'true' ]
}; then
  __ush_at 43 'std::process::exit $cli_status'
  ush_fn_std__process__exit "${__ush_binding_36_cli_status}"
fi
__ush_at 45 'sudo $archive_dir/report-inventory before-install'
__ush_value_14="${__ush_binding_16_archive_dir}"
command sudo "${__ush_value_14}"'/report-inventory' 'before-install'
__ush_at 46 'cd "tests" {'
__ush_cwd_15=$PWD
ush_fn_std__process__chdir 'tests' || { __ush_status=$?; __ush_report "$__ush_status"; exit "$__ush_status"; }
  __ush_at 47 'apt_retry vp exec playwright install --with-deps chromium'
  command vize-ci-apt-retry 'vp' 'exec' 'playwright' 'install' '--with-deps' 'chromium'
cd "$__ush_cwd_15" || { __ush_status=$?; __ush_report "$__ush_status" 'cannot return to the directory the cd block started in'; exit "$__ush_status"; }
