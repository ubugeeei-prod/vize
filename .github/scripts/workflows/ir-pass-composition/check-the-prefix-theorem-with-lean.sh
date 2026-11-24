#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/ir-pass-composition/check-the-prefix-theorem-with-lean.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
curl -fsSL --retry 3 -o lean.tar.zst https://github.com/leanprover/lean4/releases/download/v4.33.1/lean-4.33.1-linux.tar.zst
echo '890afd185370f85666025b883914ab4f4b339136f8c96167b69cfb62aecaf235  lean.tar.zst' | sha256sum --check
tar --zstd -xf lean.tar.zst
lean-4.33.1-linux/bin/lean -DwarningAsError=true "$GITHUB_WORKSPACE/tests/formal/template-lint/Selection.lean" | tee theorem.log
grep -Fx "'Davinci.TemplateLint.fold_prefix' depends on axioms: [propext]" theorem.log
grep -Fx "'Davinci.TemplateLint.labels_exact' depends on axioms: [propext, Classical.choice, Quot.sound]" theorem.log
