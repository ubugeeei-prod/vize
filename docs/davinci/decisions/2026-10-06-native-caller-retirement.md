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
and before transport recovery/retry. Cleanup already entered during retirement
still finishes; synchronization, notification drains, release and shutdown never
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
from beginning recovery, complete already-entered cleanup, and restore lexical
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
