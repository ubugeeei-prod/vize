#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/fuzz/create-or-update-fuzz-triage-fix.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
title="fix(fuzz): ${VIZE_MATRIX_TARGET} found a reproducer"
run_url="${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID}"
artifact_dir="tests/fuzz/artifacts/${VIZE_MATRIX_TARGET}"
# libFuzzer writes slow units on otherwise successful campaigns too.
# Keep triaging their parser cost, but do not report them as crashes.
if compgen -G "$artifact_dir/crash-*" > /dev/null; then
  reproducer_kind="crash reproducer"
elif compgen -G "$artifact_dir/timeout-*" > /dev/null; then
  reproducer_kind="timeout reproducer"
elif compgen -G "$artifact_dir/oom-*" > /dev/null; then
  reproducer_kind="OOM reproducer"
elif compgen -G "$artifact_dir/leak-*" > /dev/null; then
  reproducer_kind="leak reproducer"
elif compgen -G "$artifact_dir/slow-unit-*" > /dev/null; then
  reproducer_kind="slow unit"
else
  reproducer_kind="reproducer artifact"
fi
body_file="$(mktemp)"
cat > "$body_file" <<EOF
Fuzz target \`${VIZE_MATRIX_TARGET}\` produced a ${reproducer_kind}.

- Workflow run: ${run_url}
- Artifact: \`fuzz-reproducers-${VIZE_MATRIX_TARGET}\`
- Commit: \`${GITHUB_SHA}\`

Download the artifact from the workflow run and replay it locally with:

\`\`\`sh
cd tests/fuzz
cargo +nightly fuzz run ${VIZE_MATRIX_TARGET} artifacts/${VIZE_MATRIX_TARGET}/<reproducer>
\`\`\`
EOF

existing_fix="$(
  gh issue list \
    --state open \
    --search "${title} in:title" \
    --json number \
    --jq "$(<"$VIZE_CI_FILES/first-pr-number.jq")"
)"

if [ -n "$existing_fix" ]; then
  gh issue comment "$existing_fix" --body-file "$body_file"
else
  gh issue create --title "$title" --body-file "$body_file"
fi
