#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/style-module-products/verify-source-coordinates-and-style-headers.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
cd "$RUNNER_TEMP"
curl -fsSL --retry 3 -o lean.tar.zst https://github.com/leanprover/lean4/releases/download/v4.33.1/lean-4.33.1-linux.tar.zst
echo '890afd185370f85666025b883914ab4f4b339136f8c96167b69cfb62aecaf235  lean.tar.zst' | sha256sum -c
tar --zstd -xf lean.tar.zst
lean-4.33.1-linux/bin/lean -DwarningAsError=true "$GITHUB_WORKSPACE/tests/formal/source-map/LineBreaks.lean" | tee line-breaks.log
grep -Fx "'Davinci.SourceMap.LineBreaks.first_exact' does not depend on any axioms" line-breaks.log
grep -Fx "'Davinci.SourceMap.LineBreaks.last_exact' depends on axioms: [propext]" line-breaks.log
lean-4.33.1-linux/bin/lean -DwarningAsError=true "$GITHUB_WORKSPACE/tests/formal/source-map/Coordinates.lean" | tee theorem.log
grep -Fx "'Davinci.SourceMap.column_exact' depends on axioms: [propext, Quot.sound]" theorem.log
grep -Fx "'Davinci.SourceMap.diagnostic_range_rebased' depends on axioms: [propext, Quot.sound]" theorem.log
lean-4.33.1-linux/bin/lean -DwarningAsError=true "$GITHUB_WORKSPACE/tests/formal/style-format/Header.lean" | tee header.log
grep -Fx "'Davinci.StyleFormat.accepted_iff' depends on axioms: [propext, Classical.choice, Quot.sound]" header.log
