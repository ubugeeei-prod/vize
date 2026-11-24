#!/usr/bin/env bash
set -euo pipefail
curl -fsSL --retry 3 -o "$RUNNER_TEMP/lean.tar.zst" https://github.com/leanprover/lean4/releases/download/v4.33.1/lean-4.33.1-linux.tar.zst
echo "890afd185370f85666025b883914ab4f4b339136f8c96167b69cfb62aecaf235  $RUNNER_TEMP/lean.tar.zst" | sha256sum --check --strict
tar --zstd -xf "$RUNNER_TEMP/lean.tar.zst" -C "$RUNNER_TEMP"
lean="$RUNNER_TEMP/lean-4.33.1-linux/bin/lean"
"$lean" -DwarningAsError=true Emission.lean | tee "$RUNNER_TEMP/literal-axioms"
grep -Fx "'Davinci.StringLiteral.literal_round_trip' depends on axioms: [propext, Quot.sound]" "$RUNNER_TEMP/literal-axioms"
"$lean" -DwarningAsError=true --run Emission.lean "$RUNNER_TEMP/rust-literals.jsonl"
node "$VIZE_CI_FILES/check-emitted-literal-values.mts" "$RUNNER_TEMP/rust-literals.jsonl"
printf '%s\n' '["x","\"y\""]' > "$RUNNER_TEMP/wrong-literal.jsonl"
if "$lean" --run Emission.lean "$RUNNER_TEMP/wrong-literal.jsonl" > "$RUNNER_TEMP/literal-rejection" 2>&1; then
  exit 1
fi
grep -F 'different emitted spelling at row 1' "$RUNNER_TEMP/literal-rejection"
