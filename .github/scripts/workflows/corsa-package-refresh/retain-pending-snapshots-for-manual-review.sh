#!/usr/bin/env bash
directory="$RUNNER_TEMP/projection-import-geometry/pending-snapshots"
mkdir -p "$directory"
find crates/vize_maestro/tests/snapshots -name '*.snap.new' -exec cp {} "$directory/" \;
