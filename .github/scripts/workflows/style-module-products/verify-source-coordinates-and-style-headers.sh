#!/usr/bin/env bash
set -euo pipefail
cd "$RUNNER_TEMP"
curl -fsSL --retry 3 -o lean.tar.zst https://github.com/leanprover/lean4/releases/download/v4.33.1/lean-4.33.1-linux.tar.zst
echo '890afd185370f85666025b883914ab4f4b339136f8c96167b69cfb62aecaf235  lean.tar.zst' | sha256sum -c
tar --zstd -xf lean.tar.zst
lean-4.33.1-linux/bin/lean -DwarningAsError=true "$GITHUB_WORKSPACE/tests/formal/source-map/Coordinates.lean" | tee theorem.log
grep -Fx "'Davinci.SourceMap.column_exact' depends on axioms: [propext, Quot.sound]" theorem.log
lean-4.33.1-linux/bin/lean -DwarningAsError=true "$GITHUB_WORKSPACE/tests/formal/style-format/Header.lean" | tee header.log
grep -Fx "'Davinci.StyleFormat.accepted_iff' depends on axioms: [propext, Classical.choice, Quot.sound]" header.log
