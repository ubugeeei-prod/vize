# Retain each bind-style capture separately

The original bind-style observer at signed main
`6a75086cf305ea576117cc48ec68d1ad6a6f2cdd` writes into the fixed
`target/differential/lsp-bind-style-code-actions/literal-original` and
`authored-positive` directories. The Rust target cache retains those directories.
Its unchanged one-process oracle counts every `process-*` child, including any
earlier invocation's records. Recursive directory creation does not make that
evidence belong to the current invocation.

Each invocation now creates one fresh `invocation-*` directory under the same
artifact root. Its original project, CLI comparison, literal and authored wire
captures, and `result.json` all remain together inside that directory. Previous
fixed leaves and previous invocation directories remain intact. The existing
recursive corpus upload includes the new directories without workflow changes.

This only changes test evidence placement. The original client and passive
observer, exact diagnostics and whole responses, closure checks, one-process
oracle, binary identity, provider preparation and timeouts are unchanged. No
product pipeline stage, runtime allocation, cache policy or release gate changes.

## Regression proof

The filesystem regression retains two old process records and the old result.
The fixed-leaf selection sees both records. Two consecutive invocations receive
different empty directories and each keeps its own literal and authored leaves,
even when its process identifier repeats. The test compares all retained old
bytes and each new leaf's complete membership.

The initial `c5affe45` regression executed the live output-placement paragraph:
the unchanged signed-main paragraph failed and fresh invocation ownership
passed under Node 24.14.0. Its source Check `37883268959` passed; the actual
whole original stdio law passed in 12,499.021642 ms. That is historical source
evidence, not protected merge or current-main acceptance.

Review #4226596355 identified formatting-sensitive source slicing. One callable
helper now owns configured/default parent selection and fresh directory creation;
the unit exercises both branches directly without source slicing or VM evaluation.
All retained old bytes and consecutive literal/authored membership checks remain.
The revised helper and law pass Node 24 and strict native TypeScript locally.
The original whole stdio law and
fresh exact-head Actions, protected merge, full current-main qualification and
successful release remain required; the filesystem test grants none of them.

## Original failure and limits

[Check 37806185873, full scripts job 113411731434](https://github.com/ubugeeei-prod/vize/actions/runs/37806185873/job/113411731434)
failed the literal capture's `children.length == 1` assertion with actual `2`.
The earlier complete four-line diagnostic/action-title assertion had passed.
The whole retained log is 1,922,999 bytes, SHA-256
`c25edcc42ab1b4c7a5f7a20a6615e42faedd98812011bf16c956511093758ec7`.

The unexpired original artifact `11564441980` belongs to that exact main head:
131,612,216 bytes, SHA-256
`e7493f42c2d1ddaa59a0eae07d0b4e7d1eaeb3d6402229de4a22ac918d631f71`.
Until its two process receipts are extracted and compared, their actual owners
and the historical failure's cause remain unclassified. The structural reuse
defect and deterministic regression justify isolation independently of that
causal claim.

The same full job also returned PTY helper status `124` at its unchanged
25-second deadline. Its stalled phase remains unknown. The unchanged real
release prompt/abort/no-mutation law naturally passed in
[source job 113428766194](https://github.com/ubugeeei-prod/vize/actions/runs/37810962192/job/113428766194)
in 1,009.471509 ms. That is a control observation, not a current-main repair or
full-check success. Preserve both timeouts and require fresh current-main
qualification before preparing an immutable release source.

This decision is paired with #6239 and #6830; neither issue is closed by this
test evidence correction.
