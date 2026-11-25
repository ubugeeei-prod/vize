#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/crates/vize_canon/tests/programs/report-native-outcome.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
directory=${0%/*}
if [ -f "$directory/fail-shard-one" ]; then
  case "$*" in
    *tsconfig.shard1.json*) printf 'native unavailable'; exit 41 ;;
  esac
fi
cat "$directory/native-output"
if [ -f "$directory/native-error" ]; then cat "$directory/native-error" >&2; fi
exit "$(cat "$directory/native-status")"
