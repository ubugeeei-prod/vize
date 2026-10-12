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
The [official signal policy](https://sourceware.org/gdb/current/onlinedocs/gdb.html/Signals.html)
defines `noprint` as implying `nostop`; use `stop print pass` and verify the
actual bounded `info signals SIGSEGV` row before attaching. Missing or changed
stop/pass evidence refuses attachment. Printed startup remains excluded.
Pass SIGCONT without stopping, verifying its actual No/No/Yes policy before
attach. Require an authenticated actual continued-inferior event before authored
release/cancellation; capture actual exit events once, excluding detach events.
Continuation output remains in the private pipe with non-marker text hash-only.
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

The reviewed corrective driver `7b5db392372dc5d21cad78b664c1061b0e156b42`
ran [38184042465](https://github.com/ubugeeei-prod/vize/actions/runs/38184042465),
attempt 1, job `114602459092`, against the same fresh SDK source. Actual policy,
private actor matching and attachment were retained. Its normal control stopped
unexpectedly at the diagnostic SIGCONT, returned original status 0 without an
actual inferior exit marker, and was incorrectly admitted by the old control
law. Cancellation retained actual debugger reaping and pidfd resume but required
forced escalation; it failed. Fault/original-SIGINT controls and the original SDK
runner did not run; all 29,059 physical source bodies remained equal. Preserve
this failure, require actual exit/clean cancellation proof, and record each
bounded cleanup signal attempt/wait result without waiving escalation. The
earlier unmarked startup cause remains unknown. Artifact `11718886828` has SHA256
`2fa480bb9f7247b08ce9655f0ff73c56222e3fc202cdb19ae182bf0b456582e7`.

## Third failed control and attach event admission

Fresh-main driver `152f0e72b65eaa7fa8d56562d902285190a4b0db`, tree
`b89c1b4a6a701551bdedd32a348c830aa0de5adf`, ran
[38185715066](https://github.com/ubugeeei-prod/vize/actions/runs/38185715066),
attempt 1, job `114607475966`, on historical SDK `c9a37e28dec3ed683e1c993c567d289206ef34d1`.
The normal control authenticated its private actor and actual signal policy, but
timed out without a retained continuation or exit marker. Its cleanup genuinely
retained GDB-origin SIGINT, signal discard, detach, first-attempt GDB reaping with
status 0, verified birth/TracerPid 0, pidfd resume and original toy status 0.
These cleanup outcomes do not qualify the failed normal control. The other three
cases and original SDK runner did not start; all 29,059 whole physical source
bodies remained equal. Artifact `11718804952` has SHA256
`ea1848cb49e130b16855e0f9975b35330435d45413df4407fe4b3306ed7e8a05`.
Unknown GDB stdout was hash-only and its raw bytes were not retained; the actual
debugger version and attach-time callback state were absent. Preserve the third
readiness failure's cause as unknown.

The official GNU `gdb-15.1-release` source at commit
`7b20890ad616baf811c8fe0e5d2bbb54b7c8ac42` demonstrates a separate concrete
admission defect: Linux attach marks newly found LWPs running, notifying the
Python continuation registry. The old helper connected that registry before
attach, consumed first readiness before assigning its held inferior, and captured
attach command output. An authored event law reproduces lost readiness against
the complete old helper and passes after the fix; it does not identify the old
runner's installed GDB or substitute for an actual Linux control.

| Official GNU source                                                                                                       | Complete body SHA256                                               |
| ------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| [Linux attach](https://gnu.googlesource.com/binutils-gdb/+/refs/tags/gdb-15.1-release/gdb/linux-nat.c)                    | `7bb343ace09e294c8bd0506a7366d2fd0b9a1e4c492e8de2c483b2d33e2a52ab` |
| [Thread resume notification](https://gnu.googlesource.com/binutils-gdb/+/refs/tags/gdb-15.1-release/gdb/thread.c)         | `8ffce32ee8f725631e77f426bf2a3ff40905514b28aa0869c65b3e0665d7e1f7` |
| [Python resume dispatch](https://gnu.googlesource.com/binutils-gdb/+/refs/tags/gdb-15.1-release/gdb/python/py-inferior.c) | `94a4d22d2ecc485fc5fa6b41ac25c246175c0104584cde463f6f4dd273c89753` |

Ignore attach and pre-command events for readiness, retaining bounded counts.
Only the actual continuation event of the authenticated held inferior after its
explicit resume request may publish first readiness. Existing fault/provider
and original-signal checks remain independent. Retain actual debugger ELF/body,
closed version projection and before/after equality, callback counts and strict
marker framing. Keep at most 128 KiB of each raw stdout/attach transcript in its
private directory with mode 0600, excluded from every upload; publish only byte
counts, hashes and truncation flags. Missing provider or owned continuation proof
refuses control qualification. Further Linux capture still requires review.

## Fourth controlled lifecycle packet

[Run 38187662762](https://github.com/ubugeeei-prod/vize/actions/runs/38187662762),
attempt 1, job 114613407883, used driver
`e912e65f519b20f6583d73e0ecdf16b7792f90ab` and fresh SDK source
`b4f7bab24f8ecbf4aa07bfd0fc60e8c7bcaf74a3` (tree
`7f98c07fb7155f67d1ebe8ab8742377fa68c3fba`). It failed; the original SDK runner
never started. All 29,128 physical source records, 239,021,229 body bytes, match
before and after. Retain all four historical failure packets.

Actual GDB 15.1 ELF/body SHA256
`3832cc070ae1716e322105d3b39fb398695e5f031c9d39224cf227a8c2b889f6`
matched before/after. Normal and cancellation controls passed: attach callbacks
were counted separately, genuine owned continuation was admitted, normal exit
was observed, and cancellation proved origin/discard/detach/tracer-zero/resume
and clean debugger reaping. The SIGSEGV control captured its faulting thread/PC
and stable physical providers, then reported `PostFaultStop` and `PassSignalerror`
without an actual exit marker. Its post-stop kind/signal and GDB error text were
not retained; the cause remains unknown. Original SIGINT did not run. These
partial controls supply no SDK qualification, repair, performance or release
credit, and no further capture is authorized.

Artifact 11720551608 is a complete 4,316,621-byte ZIP with SHA256
`4c04f8e5590529797b0b0b6503b96b88382ef60373bf4ca5b8e6f6b3b40c2e21`.

### Single-fault onward delivery preparation

The immutable [Node 24.14.0 signal implementation](https://github.com/nodejs/node/blob/f657bb8ed86365ff3fdbe32e27563e778b41486a/src/node.cc)
(body SHA256 `01d3ea16fd426f98a13c2bba6df787ee363b677291c21588274d78b50f28927e`)
can re-raise SIGSEGV from its trap handler and SIGINT from `SignalExit`. This
supports a counterfactual against the helper's immediate-exit assumption; it
still does not identify the fourth packet's unobserved second stop.

After the one captured owned fault and physical-provider acknowledgement,
reverify the owned inferior, set only that original signal to `nostop noprint
pass`, and require the actual bounded policy row `No/No/Yes` before continuation.
Use the same verified onward policy when forwarding a genuine original signal
in cleanup. Initial SIGSEGV `Yes/Yes/Yes` admission stays unchanged. A proven
diagnostic SIGINT still follows discard/detach/resume. This is a single-fault
capture budget, allowing runtime re-raise without repeated capture or invented
exit evidence; an actual exit event remains required. Authored re-raise laws
fail against the complete published helper and pass after this change. Preserve
measured toy status before qualification refuses it; missing/changed onward
policy stays red. Further Linux execution remains held for concrete review.

## Actual four-case lifecycle qualification

[Run 38188634684](https://github.com/ubugeeei-prod/vize/actions/runs/38188634684),
attempt 1, job 114616352497, completed successfully in 41 seconds at driver
`8e962e4f6c0fbef9da5ac9d1d1d21191050f9b10` (tree
`2b5fb51305938ac04a93fabbbf6efe0fa3a7af2b`), against fresh source
`b4f7bab24f8ecbf4aa07bfd0fc60e8c7bcaf74a3` (tree
`7f98c07fb7155f67d1ebe8ab8742377fa68c3fba`). All 29,128 physical source records,
239,021,229 body bytes, independently match before/after; the SDK runner never
started. Preserve all four prior failed packets and their unknown causes.

| Authored control | Actual terminal evidence                                                                                          |
| ---------------- | ----------------------------------------------------------------------------------------------------------------- |
| Normal           | Owned continuation, actual exit event, original runner status 0.                                                  |
| Cancellation     | Authentic debugger SIGINT origin, discard/detach, tracer 0, owned pidfd resume, clean reaping, original status 0. |
| SIGSEGV          | One faulting thread/PC, stable providers, verified onward pass, actual exit event, original status 1.             |
| Original SIGINT  | Origin distinct from debugger, no discard, verified onward pass, actual exit event, original status 1.            |

Actual GDB 15.1 ELF/body matches before/after in every case. The frozen helper
bodies match the driver exactly. Attach callback counts are 11/10/10/10; each
case admits one owned readiness event after explicit resume. All marker framing
matches packet counts; no source/provider/reader gaps or forced debugger exit
occurred. Raw transcripts remain private and excluded from uploads.

Artifact 11720309069 is a complete 4,322,226-byte ZIP with SHA256
`48a4de7a022d66b67d330cf045de672b3cf36f31361b8d3052129a031bcb4e61`.
This qualifies the authored diagnostic lifecycle only. It supplies no unchanged
SDK qualification, actual SDK fault attribution, timing or release credit.
A separate reviewed authorization is required before original SDK capture.

## Integration after the artifact/Rust prefix

The independent peer verified the fifth four-case packet, and ordinary source
Check 38190379826 passed all 30 jobs at `fc185b064ccf15a1cca9c8b6420aebb9891c06cf`.
After the artifact/Rust Stack actually merged at
`13d2fb3c365d2bfc01773dab132970a65f6bfc4d`, retain every incoming job, input and
qualification clause while replaying the two explicit worker choices. The
diagnostic helper bodies remain byte-exact; source Actions must qualify the
composed caller again. This introduces no new controlled or SDK execution.

## Qualification and remaining work

This diagnostic changes worker scheduling and adds a profile probe. It supplies
no unchanged-run qualification, repair, performance or release credit. Original
assertions, client budgets, native recipes and default CI remain unchanged.
Meaningful actor/PID reuse/ESRCH/marker/privacy/partial attach/fault pass/cancellation
controls run in the ordinary tooling gate. A normal control exit or unavailable
capture does not explain or repair the original SIGSEGV.

TODO: obtain separate authorization for original SDK capture; retain its
original runner status and
finite gaps, attribute only actual fault/provider evidence, and obtain ordinary
fresh-source full qualification independently. No blind retry of the historical
failed source is authorized by this component.
