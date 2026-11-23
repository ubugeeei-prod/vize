#!/usr/bin/env bash
for attempt in 1 2 3; do
  cargo fetch --locked --target "$(rustc -vV | sed -n 's/^host: //p')" && exit 0
  echo "::warning title=crates.io fetch::attempt $attempt failed"
  sleep $((attempt * 30))
done
exit 1
