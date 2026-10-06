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
