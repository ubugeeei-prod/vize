#!/bin/bash
# resolve-rebase.sh : during a deletion-PR rebase, keep our deletions, take main's generated
# indexes (regenerated afterwards), and stop on anything else.
set -e
for i in $(seq 1 20); do
  u=$(git diff --name-only --diff-filter=U)
  [ -z "$u" ] && { git rev-parse -q --verify REBASE_HEAD >/dev/null 2>&1 || exit 0; }
  for f in $u; do
    st=$(git status --porcelain -- "$f" | cut -c1-2)
    case "$st" in
      UD|DU) git rm -q -- "$f" ;;   # deleted by the PR, modified on main: keep the deletion
      *) case "$f" in
           .github/workflows/README.md|tests/tooling/fixtures/tooling-test-timings.json|tools/config/retired-tooling-languages.json|docs/release/v1-supported-matrix.json)
             git checkout --ours -- "$f"; git add -- "$f" ;;
           *) echo "MANUAL $f"; exit 1 ;;
         esac ;;
    esac
  done
  GIT_EDITOR=true git rebase --continue >/dev/null 2>&1 || true
  git rev-parse -q --verify REBASE_HEAD >/dev/null 2>&1 || [ -n "$(git diff --name-only --diff-filter=U)" ] || exit 0
done
