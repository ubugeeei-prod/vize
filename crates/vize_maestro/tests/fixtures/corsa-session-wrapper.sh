#!/bin/sh
wrapper_dir=$(pwd -P)
trace_dir="$wrapper_dir/protocol-traces"

if [ -f "$trace_dir/shutdown-gate.port" ]; then
  gate_port=$(cat "$trace_dir/shutdown-gate.port")
  perl "$wrapper_dir/trace-client.pl" "$trace_dir/client-$$.raw" "$trace_dir/shutdown-gate.enabled" "$gate_port" | "$wrapper_dir/actual-tsgo" "$@" 2>"$trace_dir/server-$$.stderr"
else
  tee "$trace_dir/client-$$.raw" | "$wrapper_dir/actual-tsgo" "$@" 2>"$trace_dir/server-$$.stderr"
fi

status=$?
printf '%s\n' "$status" >"$trace_dir/process-$$.reaped"
exit "$status"
