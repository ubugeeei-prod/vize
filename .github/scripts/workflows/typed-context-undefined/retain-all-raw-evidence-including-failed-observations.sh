#!/usr/bin/env bash
directory="$RUNNER_TEMP/typed-context-undefined"
mkdir -p "$directory"
mkdir -p "$directory/pending-legacy-snapshots"
find source/crates/vize_maestro/tests/snapshots -name '*.snap.new' -exec cp {} "$directory/pending-legacy-snapshots/" \;
for phase in before source; do
  for name in typed-context-undefined-10667 typed-context-undefined-options-10667; do
    path="$phase/target/vize-tests/$name"
    if [[ -d "$path" ]]; then
      mkdir -p "$directory/$phase"
      cp -a "$path" "$directory/$phase/"
    fi
  done
done
