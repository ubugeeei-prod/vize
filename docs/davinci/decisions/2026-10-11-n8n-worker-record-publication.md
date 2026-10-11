# Atomic n8n worker record publication

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The original n8n replay prerequisite on UI PR #8555 failed in
[Check 38115321297](https://github.com/ubugeeei-prod/vize/actions/runs/38115321297),
job 114399004844: 30 of the original 31 input laws passed. The sole failure was
the normally closed worker's ignored-stdio descendant law. Its sibling read
`worker-process.json` while the parent truncated and rewrote that marker;
`JSON.parse` received empty bytes, so the sibling exited 1 instead of the
original required 7. The whole untouched `input-laws.json` packet has SHA256
`b7b3298cf62346c1d63216065942c71ea6fa390ced3b6fe36368c9e5f40bf1de`.

Publish the exact original JSON bytes through an exclusive temporary file in
the same owned output filesystem and an atomic rename. A concurrent reader
sees the previous complete packet or its complete replacement. A write,
rename or cleanup failure still reaches the original failure classification
and group termination. Remove the owned staging directory after each attempt.

Keep every original record field, byte stream, process/group ownership,
assertion and the original 2-second law, 900-second campaign and shared 32MiB
bounds. Add deterministic controls which read the actual marker between a
real partial staged write and its completion, and which fail an actual staged
write while requiring the prior whole packet and staging cleanup. The original
four-file law entry point executes these controls alongside all 31 original
laws. Exact-head Actions and protected queue evidence remain required.
