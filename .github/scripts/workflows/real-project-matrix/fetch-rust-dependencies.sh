#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/real-project-matrix/fetch-rust-dependencies.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
for attempt in 1 2 3; do
  cargo fetch --locked --target "$(rustc -vV | sed -n -f "$VIZE_CI_FILES/../../../../tools/ci/lib/scripts/rust-host-triple.sed")" && exit 0
  echo "::warning title=crates.io fetch::attempt $attempt failed"
  sleep $((attempt * 30))
done
exit 1
