# Validated warm LSP query surfaces

Issue: [#8008](https://github.com/ubugeeei-prod/vize/issues/8008).
Paired decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/8008#issuecomment-5993441802).

The current warm request path validates an AliasContext and then reconstructs the host/dependency query surface, including import walks, materialized source maps and document packets. Overlapping fingerprint and project-member validation also reads the same disk inputs repeatedly within one lookup. This is a source-level explanation to test, not an accepted timing or CPU result.

Prepare a bounded immutable query surface inside that existing AliasContext revision. Reuse it only after the unchanged host/overlay/options/request-membership fingerprint and strong closed-file/configuration/project-member validation succeeds. Use one temporary InputStampCache only inside each individual cache lookup; discard it before any build/materialization and before the next lookup. Preserve same-mtime/same-length content and missing/config/package route checks. Do not use received watcher generations as proof that external files are unchanged.

The normal materialized project, virtual-document synchronization, native request/readiness/recovery, cancellation and stale-response guards remain in place. This caches generated input data, not native responses. Full public owned packets still clone, and distinct closed disk inputs still receive strong content validation. The existing eight-context bound remains; no new stage, IPC, process or persistent filesystem-observation cache is introduced.

Retain the complete original issue body and exact generator, including all 400 SFCs and 134 TS modules for the reported large case. The package/config files materialize options described by the reporter; they are not presented as literal files supplied in the report. Whole cached-versus-uncached host/dependency/materialized/map/document packets and closed dependency/config same-mtime/same-length edits, unsaved overlays, close/reopen, source options, requested membership, cross-root identities and fresh mirror-session ownership are source laws. Existing native cancellation/recovery/refusal guards and their complete laws remain unchanged.

TODO: reuse the existing Canon scaling Actions compare job for source-qualified before/head builds on one worker, identical dependencies and the original 400 generator/workspace, five rounds of the four reported requests in each warm process, whole raw client/server/results/exit packets and Linux process-tree CPU observations. Actual dependency versions must be disclosed separately from the report. No local native build, private benchmark, speedup, native-default/history completion or release acceptance is claimed. Fresh source Actions, the paired current-source observation, protected full suites/all unchanged instruction ceilings, actual reporter-credited merge and coordinated publication remain required.

The original report and generator SHA-256 values are respectively
`247d435038eb8c53f317dc6a5fdcdd216a19fa9bc58d155e355b5ed2c3eb4a58` and
`4d0e14d66ce6e7b0b5d2e32a51c39bbe985283aa94934616b4da0ff587d34649`.
The reported dependency/platform/timing observations remain historical, with
no transfer to current source execution or CPU measurements.

The [bounded source review checkpoint](https://github.com/ubugeeei-prod/vize/issues/8008#issuecomment-5993558094)
clears production4d3c only. Compare canonical physical temporary roots in the
complete cross-root membership law, including macOS `/var` symlink spelling.
The fixture names its current Rust source law; the original400 paired whole
wire/native-process/currentCPU observer remains pending. Session config PATH
equality is not whole native configuration execution, and a fresh mirror
session is not an actually executed new native process. Source checks remain
mandatory on the corrected head, with no earlier runtime or timing transfer.
