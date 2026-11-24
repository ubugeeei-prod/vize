#!/usr/bin/env bash
# Evidence for the quiet-crate pilot. Run from the pilot worktree.
#   pilot-evidence.sh static   token identity, counters, lengths, fmt (no build)
#   pilot-evidence.sh build    tests + CI clippy for the six crates (needs >= 60 GB free)
#   pilot-evidence.sh ir       release-profile IR identity, locations normalized (fleet lock)
set -euo pipefail
S=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/9220dd2e-014b-44eb-93b7-7a91fbaceb45/scratchpad
TOOLS=$S/rd-l0
OUT=$S/rd-evidence
mkdir -p "$OUT"
CRATES=(vize_fresco vize_musea vize_doctor vize_marquette vize_curator vize_armature)
PKGS=(); for c in "${CRATES[@]}"; do PKGS+=(-p "$c"); done
free_gb() { /bin/df -g /Users | tail -1 | awk '{print $4}'; }

case "${1:-static}" in
static)
  # The one moved file is the test rename (class T); everything else must pass N0.
  node "$TOOLS/tools/commands/levels/rust-token-identity.ts" --base origin/main > "$OUT/token-identity.txt" || true
  head -1 "$OUT/token-identity.txt"; grep -E "^  (different|added|deleted|moved|position|doc-product)" "$OUT/token-identity.txt" || true
  node "$TOOLS/tools/commands/ci/source-file-lengths.ts" --check --base-ref origin/main | tail -1 | tee "$OUT/lengths.txt"
  files=$(git ls-files "${CRATES[@]/#/crates/}" | grep '\.rs$')
  node "$TOOLS/tools/commands/ci/readability-ratchet.ts" --files $files > "$OUT/counts-head.txt"
  tmp=$(mktemp -d); git archive origin/main "${CRATES[@]/#/crates/}" | tar -x -C "$tmp"
  (cd "$tmp" && node "$TOOLS/tools/commands/ci/readability-ratchet.ts" --root "$tmp" --files $files > "$OUT/counts-base.txt"); rm -rf "$tmp"
  for f in base head; do
    awk -v side=$f '{for(i=2;i<=NF;i++){split($i,a,"=");s[a[1]]+=a[2]}} END{printf "%s:", side; for(k in s) printf " %s=%s", k, s[k]; print ""}' "$OUT/counts-$f.txt"
  done | tee "$OUT/counts.txt"
  ;;
build)
  [ "$(free_gb)" -ge 60 ] || { echo "only $(free_gb) GB free; wait"; exit 2; }
  nix develop --command bash -c "CARGO_TARGET_DIR=\$PWD/target cargo test --locked ${PKGS[*]} 2>&1 | tail -n 400" > "$OUT/test.log"
  grep -E "^test result|FAILED|panicked" "$OUT/test.log" | sort | uniq -c | tee "$OUT/test-summary.txt"
  nix develop --command bash -c "CARGO_TARGET_DIR=\$PWD/target cargo clippy --locked ${PKGS[*]} --profile ci -- -D warnings -D clippy::wildcard_imports" > "$OUT/clippy.log" 2>&1 && echo "clippy clean" | tee -a "$OUT/test-summary.txt"
  rm -rf target/debug/incremental target/ci/incremental
  ;;
ir)
  [ "$(free_gb)" -ge 60 ] || { echo "only $(free_gb) GB free; wait"; exit 2; }
  lock=/private/tmp/claude-501/vize-release-build.lock
  mkdir "$lock" || { echo "release lock held: $(ls -ld "$lock")"; exit 3; }
  trap 'rmdir "$lock"' EXIT
  crates=(); for c in "${CRATES[@]}"; do crates+=(--crate "$c"); done
  nix develop --command bash -c "node $TOOLS/tools/commands/levels/ir-identity-locations.ts --base origin/main ${crates[*]} --work $S/rd-ir-work --diff 4" | tee "$OUT/ir-identity.txt"
  rm -rf "$S/rd-ir-work"
  ;;
esac
