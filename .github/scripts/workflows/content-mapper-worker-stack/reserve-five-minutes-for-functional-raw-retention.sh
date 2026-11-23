#!/usr/bin/env bash
mkdir -p "$RUNNER_TEMP/mapper-admission"
deadline="$(( $(date -u +%s) * 1000 + 15 * 60 * 1000 ))"
printf 'ADMISSION_DEADLINE_MS=%s\n' "$deadline" >> "$GITHUB_ENV"
printf '{"hardMinutes":20,"softDeadlineMs":%s,"artifactReserveMinutes":5,"performance":false}\n' "$deadline" > "$RUNNER_TEMP/mapper-admission/budget.json"
