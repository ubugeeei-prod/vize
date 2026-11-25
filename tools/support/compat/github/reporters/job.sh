#!/bin/bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tools/support/compat/github/reporters/job.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
# Cached tasks can filter environment variables and still inherit this executable on PATH.
tool=@@VIZE_REPORT_TOOL@@
capture_environment=(@@VIZE_REPORT_ENV@@)
for entry in "${capture_environment[@]}"; do export -- "$entry"; done

case "$tool" in
  cargo)
    if [[ "${1-}" == "test" ]]; then exec "$VIZE_JOB_TEST_CASE_NODE" "$VIZE_JOB_TEST_CASE_TOOL" exec-cargo -- "$@"; fi
    exec "$VIZE_JOB_TEST_CASE_CARGO" "$@"
    ;;
  vp)
    if [[ "${1-}" == "test" && "${2-}" == "run" ]]; then exec "$VIZE_JOB_TEST_CASE_NODE" "$VIZE_JOB_TEST_CASE_TOOL" exec-vp -- "$@"; fi
    exec "$VIZE_JOB_TEST_CASE_VP" "$@"
    ;;
esac

script=""
previous=""
after=()
for argument in "$@"; do
  if [[ -n "$previous" ]]; then previous=""; continue; fi
  if [[ -z "$script" ]]; then
    case "$argument" in
      -e|--eval|-p|--print|-r|--require|--import|--loader|--experimental-loader|--test-reporter|--test-reporter-destination)
        previous=1
        continue
        ;;
    esac
    if [[ "$argument" == "--test" ]]; then exec "$VIZE_JOB_TEST_CASE_NODE" "$VIZE_JOB_TEST_CASE_TOOL" exec-node -- "$@"; fi
    if [[ "$argument" == -* ]]; then continue; fi
    script="$argument"
    continue
  fi
  after+=("$argument")
done
normalized="${script//\\//}"
base="${normalized##*/}"
capture_script=0
if [[ "$normalized" != */node_modules/* ]]; then
  if [[ "$base" =~ \.(test|spec)\.[cm]?[jt]sx?$ || "$base" =~ ^(test|tests|run-tests|run-host|host)\.[cm]?[jt]sx?$ || "$base" =~ -judge\.[cm]?[jt]sx?$ || "$normalized" == */test/* || "$normalized" == */tests/* || "$normalized" == */support/compat/parcel/run.ts ]]; then
    capture_script=1
  fi
fi
if [[ "$base" == "vp" && "${after[0]-}" == "test" && "${after[1]-}" == "run" ]]; then capture_script=1; fi
if [[ ( "$base" == "tsx" || ( "$base" == "cli.mjs" && "$normalized" == */tsx/* ) ) && " ${after[*]-} " == *" --test "* ]]; then capture_script=1; fi
if [[ "$capture_script" == 1 ]]; then exec "$VIZE_JOB_TEST_CASE_NODE" "$VIZE_JOB_TEST_CASE_TOOL" exec-node -- "$@"; fi
exec "$VIZE_JOB_TEST_CASE_NODE" "$@"
