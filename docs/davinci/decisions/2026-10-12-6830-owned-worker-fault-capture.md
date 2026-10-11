# Opt-in owned Node test-worker fault capture

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The original SDK source `f2b31f4a8f1b677629c29c1be4f4ebfb68fc800e`
failed the complete JS package runner at physical merge source
`52c78c8865237e8503a592f13029231b55da0537`, run `38161052557`,
attempt 1, job `114534383424`. The isolated
`npm/builder/vite-musea/src/variant-bindings-http.test.ts` worker died with
SIGSEGV. Its physical fault frame and loaded native provider were not retained;
the reachable import graph and freshly built formatter addon do not establish
the crashed worker's loaded provider. This failure remains a failure.

## Decision

Keep the merged ordinary Linux observer from #8686 unchanged. Its private
Firecracker requirement correctly refused kernel-wide core setup when DMI was
absent. Add separate `owned-worker-fault` and `owned-worker-controls` manual components, excluded from
`all`, PR, push, queue and scheduled events. Require an authenticated full fresh
source SHA; the historical failed source and head are explicitly refused.
Retain orchestration helper hashes, actual source head/tree and original failure
provenance before checking out that source. Run the original complete
`vp run --workspace-root test:js` argv once with its original environment and
standard streams. Two diagnostic-only environment variables are removed before
both the profile probe and original runner. Setup or observation failure cannot
convert the runner's exit or signal to success.
Copy and authenticate the merged read-only whole source census, forcing
`git --no-replace-objects` in its subprocess calls. Retain every tracked physical
body before and after the runner; missing or changed source/provider observations
make the capture partial. Aggregate hash calls, bytes and read time are retained
outside identity equality, separately in every observer/debugger process.

Before the runner, execute one authored, no-sleep Node test probe with the exact
installed Musea cwd, tsx test/reporter recipe, physical package/tsconfig and
allowlisted original Node options. Privately read its actual `/proc/self/cmdline`
and actual `NODE_TEST_CONTEXT=child-v8` value. Freeze the complete private worker
argument vector, replacing only the sole probe script with the original recipe's
relative target spelling. Independently authenticate its physical source path
and whole Git blob/body. Require the target worker's one private `NODE_OPTIONS`
entry to match the frozen allowlisted recipe and hash/length. Public packets contain physical identities and
closed match booleans; private argv and environment values are never uploaded.

Select only a live descendant of the actual unreaped original runner, with
verified parent identities, PID/start ticks, UID, physical Node executable,
package cwd, original target Git blob/body, tsx preload bodies and configuration.
A parent driver, path substring, extra argument, changed source, replaced
executable/preload/configuration, missing marker or uncertain ancestry refuses
selection. Open a pidfd, recheck birth, stop only that worker through pidfd, wait
for its actual stopped state and recheck the complete actor before attaching.

GDB independently opens and verifies a pidfd and the same private actor before
and after its numeric attach. Disable autoload and debuginfod, omit frame
arguments and entry values, trace only this inferior, and stop/pass SIGSEGV.
Publish bounded marked backtraces, registers and library identities; hash actual
executable mappings/providers before attachment and at the stopped fault before
continuing the original signal. No raw core, environment, argv, startup output,
locals or pointed-to memory is published. No `core_pattern`, sysctl, global
ptrace setting, parent driver, unrelated worker or process tree is changed.
Bind executable/provider ELF64 machine/type, mapped device/inode and stable whole
file hashes. Keep the actual faulting thread selected for PC/register capture.

Refusal, partial attach, debugger failure and cancellation attempt detach.
Resume only the original pidfd after live birth/executable verification; ESRCH
is an explicit benign observation gap. PID reuse never falls back to numeric
signalling. GDB's attach API accepts a numeric PID, so verification surrounds
that operation; this does not establish an atomic pidfd-based ptrace syscall.
Any uncertain actor or diagnostic packet is unavailable evidence.
Use a small privileged supervisor to own and reap the actual GDB child, rather
than terminating only its sudo monitor. Verify actual tracer PID zero before
resuming the worker. Parent death, interruption and bounded forced debugger
cleanup produce explicit receipts; failed or forced cleanup is partial evidence.
Discard a queued debugger cleanup SIGINT only with its owned cancellation
marker and Linux siginfo proving this actual GDB sender PID/UID and signal code;
preserve an original worker SIGINT even when cancellation is also requested.
Before touching the original worker, opted-in Linux controls exercise normal
exit, fault pass, cancellation/reaping and genuine worker SIGINT under a
cancellation marker against an authored temporary Git
fixture and the actual installed Node/tsx/GDB. Only this toy fixture disables
core output; the original runner's resource limits remain unchanged. Failed
controls disable attachment while retaining the original runner once.
The toy uses the original relative target spelling and privately reads its
actual `/proc` argument vector and child-v8 marker before STOP; require complete
byte equality with the probe profile after the one script replacement. The
controls-only component produces this packet without invoking the original JS
runner, so actual debugger custody can be reviewed before the one SDK capture.

## Qualification and remaining work

This diagnostic changes worker scheduling and adds a profile probe. It supplies
no unchanged-run qualification, repair, performance or release credit. Original
assertions, client budgets, native recipes and default CI remain unchanged.
Meaningful actor/PID reuse/ESRCH/marker/privacy/partial attach/fault pass/cancellation
controls run in the ordinary tooling gate. A normal control exit or unavailable
capture does not explain or repair the original SIGSEGV.

TODO: obtain peer review of the complete helper and controls; wait for the
existing shared source-custody guard repair and authenticated fresh SDK source;
execute one explicitly authorized full diagnostic; retain its original runner
status and finite packet gaps; attribute only actual fault/provider evidence;
then obtain ordinary fresh-source full qualification independently. No blind
retry of the historical failed source is authorized by this component.
