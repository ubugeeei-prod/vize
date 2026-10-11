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
Plan these steps in a reusable workflow, so checking out the separate frozen
SDK source cannot erase a newly introduced local composite's post-job manifest.
Keep the original physical cwd, argv, environment and source custody unchanged.
Retain bounded supervisor/script phase markers and drain late cleanup packets
before final evidence is written. On failed toy controls, verify worker birth,
tracer zero and actual debugger reaping before releasing the authored watcher
and waiting a bounded interval for its original runner to drain. A fallback kill
targets only that still-unreaped authored runner and gives no cleanup credit.
Hash GDB stderr while publishing only recognized exception types and reviewed
helper basenames/line numbers/whole-source hashes. Message text, other paths and
context remain excluded; unrecognized startup is an explicit observation gap.

## First Linux controls packet

Draft #8712 driver `a85c5a0f43bc231215548c4af53324ca2d4d4ab7`
ran controls against fresh SDK source
`c9a37e28dec3ed683e1c993c567d289206ef34d1`, run
[38182533943](https://github.com/ubugeeei-prod/vize/actions/runs/38182533943),
attempt 1, job `114597965505`. The first authored normal control failed with a
timeout; subsequent controls did not run. The original SDK runner never started.
The complete private profile matched before stopping the exact authored worker;
all 29,059 tracked physical source bodies were equal before/after. No attached
or cleanup marker was retained, so actual GDB custody and the startup cause are
unavailable. Its 1,199 stdout bytes were retained only as a hash, and return code
zero does not prove attachment or cleanup. The late-packet failure path and
local composite post-hook manifest lookup require repair before another run.
Artifact `11718068707` has whole ZIP SHA256
`b14b7b481b8b589d8f8cac75153896b80888c7536cef78e9e150881eb772d5a4`.

## Qualification and remaining work

This diagnostic changes worker scheduling and adds a profile probe. It supplies
no unchanged-run qualification, repair, performance or release credit. Original
assertions, client budgets, native recipes and default CI remain unchanged.
Meaningful actor/PID reuse/ESRCH/marker/privacy/partial attach/fault pass/cancellation
controls run in the ordinary tooling gate. A normal control exit or unavailable
capture does not explain or repair the original SIGSEGV.

TODO: peer-review the controls failure successor and qualify all four authentic
Linux lifecycle packets before the explicitly authorized full diagnostic;
retain its original runner
status and finite packet gaps; attribute only actual fault/provider evidence;
then obtain ordinary fresh-source full qualification independently. No blind
retry of the historical failed source is authorized by this component.
