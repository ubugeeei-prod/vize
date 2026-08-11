#!/bin/sh
wrapper_dir=$(pwd -P)
actual_tsgo=$(cat "$wrapper_dir/actual-tsgo.path")
for argument in "$@"; do
  if [ "$argument" = "--api" ]; then
    exec "$actual_tsgo" "$@"
  fi
done
trace_dir="$wrapper_dir/protocol-traces"
gate_port=0
if [ -f "$trace_dir/shutdown-gate.port" ]; then
  gate_port=$(cat "$trace_dir/shutdown-gate.port")
fi
perl "$wrapper_dir/trace-stdio.pl" \
  "$trace_dir/client-$$.raw" \
  "$trace_dir/server-$$.raw" \
  "$trace_dir/server-$$.stderr" \
  "$trace_dir/shutdown-gate.enabled" \
  "$gate_port" \
  "$actual_tsgo" \
  "$@"
status=$?
printf '%s\n' "$status" >"$trace_dir/process-$$.reaped"
exit "$status"
