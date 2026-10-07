# Entered native caller retirement (#7698)

The native worker already skips jobs whose caller leaves before entry. Once a
job enters, its mutable client must retain ownership until committed RPCs drain.
Add an opaque caller control only inside that synchronous worker operation. A
lexical guard restores the previous control before owner reuse and on unwind;
no foreground async TLS scope is installed. Scoped readiness threads explicitly
borrow that same captured control rather than relying on inherited TLS.

Check retirement before assigning a readiness identity and after its response
has drained. Join every started worker before returning an explicit non-transient
incomplete error. A partial barrier retains all dirty documents, topology
barriers and generation state. Check again before the following semantic request
and before replacement session/retry. A completed transient failure still retires
its old session even when the caller has left; this is mandatory cleanup, not a
replacement spawn. Its original failure and cleanup error remain in the explicit
incomplete result, whose marker takes precedence over transport-error text. Cleanup
already entered during retirement still finishes; synchronization, notification drains, release and shutdown never
consult this control. The next live caller uses the unchanged synchronization
and transport-recovery policy.

Cancellation can race with a checkpoint and allow another RPC to commit. Drain
that RPC; this does not promise atomic zero packets after cancellation. At most
sixteen readiness workers remain in flight. All 534 dirty identities and the
four original 128-notification drains remain mandatory for an uncancelled caller.
No timer, SDK fork, process kill, extra request or pipeline stage is introduced.

The controlled real-Corsa JSON-RPC socket law holds sixteen complete requests,
retires the actual BoundedWorker caller, drains responses in reverse order, then
re-ACKs all 534 identities on the same retained owner before the complete authored
hover and valid omitted-params shutdown. It checks whole responses and the exact
incomplete error, not a partial projection. This is a controlled owner/generation
model, not native Program, source installation or PID/reaping evidence. Existing
full native lifetime/hover/diagnostic laws supply their own scope of qualification.
Separate laws retain normal following-owner recovery, prevent a retired caller
from starting a replacement while completing old-session cleanup, and restore lexical
ownership after unwind.

The public400+134 whole79-query/18-notification recipe and every old input,
expected diagnostic, stamp, deadline and budget stay unchanged. Only exact owned
source paths enter its binder/trigger. The Canon census is rendered by the existing
scanner over the complete Canon cone. No private input is used.

This slice removes unnecessary work after caller retirement. It does not preempt
a still-live diagnostic transaction or prove lower uncancelled first-hover
latency. The historical release337 empty-editor cause remains unknown. Current
compile/native/full/paired/protected and actual signed delivery are pending;
Issue7698 and its complete responsiveness target remain open.

Paired decision: https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6018068203.

[Cleanup clarification](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6018281589) preserves mandatory old-session retirement after a completed transient failure, then refuses replacement work for the retired caller and retains whole first/cleanup error text as non-transient incomplete.

The [actual-parent incorporation](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6019490727) retains literal signed064, every incoming source/decision and the qualified child implementation/inputs/author records. The old62bd source/full/native/paired success remains historical; its warm/background comparisons worsened, so no uncancelled gain is claimed. The remaining child is retargeted to main and requires fresh exact-head qualification plus native Stack/protected actual delivery.

[Completed-error precedence](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6019786697) preserves a genuine completed RPC error ahead of retirement-only status, including errors from other already-started joined workers. Store retirement observations separately; all started calls drain before the original transient policy cleans the failed session, then the existing checkpoint refuses replacement. Whole single-owner and reversed-window refusal payload controls are added while the original534 retirement/recovery law and cleanup assertions remain. The inherited new shell-quote audit failure is retained; fresh runtime/full/protected qualification is pending, without historical337 cause or latency credit.

[Actual security-main incorporation](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6020569737) consumes only delivered signed ef848 after #8137 merged. Every incoming security/lock/decision byte and the current reviewed child production/input/whole contracts are retained. Prior b906 full execution is complete except its inherited audit, with all six controlled laws, original SDK-enabled three-row diagnostic parity, abandoned-hover Markdown and whole host/Vue originals passing; it remains historical for the fresh integration. Its whole79/18 shipping pair preserves all contents but worsens warm/background timing, so no uncancelled gain or historical337 cause is claimed. Fresh ordinary/native/paired/full, native Stack8128 protected gates and actual signed delivery remain required.

## Fresh-main replay

The [paired refresh decision](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6043713794) incorporates delivered, signature-verified `8e8e0754c88751296641ed2b11be644a492eb358` while preserving the original b906 source, clean unpublished a634 security-main composition and complete author/decision history. The old worktree stays intact. The final source also incorporates actual, signature-verified `8fd5ff02d8b361d795c0e49878de5404257f58bb` (#8170), which arrived during composition, retaining its complete compiler/corpus and attribute bytes without a retirement product conflict. All production/RPC/lifetime/reverse-window/534 controls remain; checked Option equality on the whole authored `value` token preserves invalid-range and different-token refusal under current strict lints. Keep every incoming binder entry plus only the original eight retirement paths, the complete current scaling workflow, original400+134/whole79/18, runtime/deadlines/budgets and the independently owned navigation capacity CAS. The existing renderer scans the full actual source for the owned inventory. Native GraphQL confirms remote Stack8128 membership; root alone admits a qualified prefix.

Historical b906 full/native/paired execution and its inherited audit failure remain retained, with worse warm/background timings and no uncancelled gain or historical337-cause claim. Fresh exact-head ordinary/native/scaling/full laws, protected gates, actual signed merge and release delivery remain pending; the pinned full-command10x target and Issue7698 remain open.

## Retained original diagnosing process

For the explicit completed Original Module checker, retain one independent native
`--lsp` process and its configured API attachment on the owning project client.
It never shares the live virtual-document editor overlay. Each call still parses
the authored configuration and full root membership with the existing refresh,
checks the complete diagnosing options before and after the raw report, and closes
its own authored overlay afterward. Put complete filesystem invalidation into the
existing before-observation snapshot RPC; do not add a request or pipeline stage.
Open the explicit project reference once and release every observed snapshot;
retain the project only for this diagnosing owner's lifetime.

Reuse requires the canonical direct native image's complete SHA-256 and the full
configuration-chain session key. Unknown launchers and scripts keep the previous
fresh-process behavior, because their bytes cannot identify a delegated runtime.
Configuration/runtime replacement retires the old owner before the new one can
answer. Errors and admission refusals also retire it without replacing the existing
typed primary error. Materialized mode, native Vue mode, root/config activation,
explicit shutdown, Drop and existing SessionMap idle eviction retire this owner;
process shutdown/reaping precedes the attached API reader join.

The additive original-diagnosing-process-7698 corpus freezes the complete source,
shared dependency, config, TS2322 raw report and exact authored range. Linux native
controls compare the physical executable, PID and kernel birth through five no-op
checks, authored leaf/shared edits and exact inverses; config-option edits compare
whole effective options and require the previous native life to be reaped. Separate
shutdown, Drop, idle-to-next-owner and materialized-mode laws observe the recorded
life disappearing, with no same-birth zombie or wrapper/init-counter credit. An
explicit source-only disabled envelope does not claim native execution; Require=1
rejects disabled or absent runtimes, and the existing official native workflow
runs the added controls with its pinned direct binary. Keep every original27+6,
SDK-native lifetime fixture, 400+134 shipping input and all104 ceilings unchanged.

Corsa 1.14 has no compiler Program-object identity. This is retained diagnosing
process/project-reference work, not proof of true NativeProgram reuse or the full
10x target. The legacy product routes remain in place while #6879/#6883 are open.
Fresh exact-source native/full/protected execution, actual merge, latency evidence
and public release acceptance remain unfinished. The [paired decision](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6047368965) records this source slice on actual signed main3e with declared Rust1.98; an incoming protected candidate after the Rust migration must independently qualify1.99. Source/historical success cannot substitute for its fresh protected proof.

The [admission correction](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6047729116) retains the originally
unadmitted exported-variable fixture, every original top-level value and its
whole expected report as a negative/historical control. The first4cc native
qualification failed before the backend: require the exact unit17 bytes50..72
UnsupportedSyntax issue and typed IncompleteFile refusal with an absent backend.
Add a separate admitted primitive-return Module, complete TS2322 return-token
report at line1 UTF16 35..41, number-return/shared-string edits and exact inverses.
Ordinary cross-platform tests require complete genuine File/projection/source
identity for both admitted versions before the Linux physical controls run.
No grammar extension, runtime credit for refused bytes, input/cap reduction or
legacy replacement is part of this correction; current native/protected/actual
delivery remains unfinished.

The [physical-owner correction](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6047965780) retains server-generated opaque
API session IDs and qualifies cross-process comparisons with the complete
executable/PID/kernel-birth life. The admittedbbd3 configuration case failed
because distinct process observations can both return api-session-1; no complete
configuration-inverse qualification is credited. Keep explicit different PID/birth,
old-life reaping, whole effective options, exact inverses and same-owner no-ops.
The two additive native workflow prefixes exit on their first failed cargo status;
all original command/phase/receipt bytes and the350-row ceiling stay unchanged.
This correction changes no production source, original fixture/answer or cap.
Fresh exact-head native/full/protected execution and actual delivery remain pending.
