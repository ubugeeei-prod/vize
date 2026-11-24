#!/usr/bin/env bash
# Merge origin/main into the PR 1 branch and regenerate the seed data from the merged tree.
# Prints whether the check passes on the merged tree before and after reseeding.
set -euo pipefail
cd /private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/9220dd2e-014b-44eb-93b7-7a91fbaceb45/scratchpad/rd-l0
git fetch -q origin
if git merge-base --is-ancestor origin/main HEAD; then echo "main already merged ($(git rev-parse --short origin/main))"; fi
git merge -q --no-edit origin/main -m "Merge origin/main into test/readability-ratchet"
echo "== before reseed"
node tools/commands/ci/readability-ratchet.ts --check 2>&1 | grep -v "^  " | cut -c1-300 || true
node tools/commands/ci/readability-ratchet.ts --write --reseed | head -1 | cut -c1-300
node tools/commands/ci/byte-custody.ts --write
echo "== after reseed"
node tools/commands/ci/readability-ratchet.ts --check 2>&1 | grep -v "^  " | cut -c1-300
node --test tests/tooling/readability-*.test.ts tests/tooling/task-shell.test.ts 2>&1 | grep -E "^ℹ (pass|fail)"
git add tools/config/readability-baseline.tsv tools/config/byte-custody.tsv
git diff --cached --stat | tail -1
