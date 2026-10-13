# Linux JS native failure evidence (#6830)

The mandatory PR JS gate at source `17bb01881b3e9721c273048d4698a1fb887c6a2e`
failed with SIGSEGV in `variant-bindings-http.test.ts`: Check
[38150949243](https://github.com/ubugeeei-prod/vize/actions/runs/38150949243),
job `114504704179`. Its actual checked-out PR merge is
`a16d50297208eb4635c824449bdbb345c462351d`, tree
`670e62ad66b6ff29b441e887bc7f62abb641c106`. The later full-source JS success
[38152433486](https://github.com/ubugeeei-prod/vize/actions/runs/38152433486)
is a separate control. It does not replace the original failure or prove a repair.
No fault frame or native provider cause has yet been established.

## Decision

The existing mandatory command `vp run --workspace-root test:js` executes exactly
once with its original argv, environment, working directory and inherited streams.
An additive Python observer records physical source/tree/diff and every tracked
source file's hash, read-only sampled Node descendant maps and each observed
executable/native provider's SHA-256, size and mapped inode. Missing, replaced or
unreadable observations stay typed gaps; sampled maps are not exhaustive call
proof. Diagnostic failures do not turn an original test failure into success.
Original native/formatter/style/Oxlint uploads, both real host qualification and
staging cleanup remain mandatory with their original guards and paths. No retry,
optimizer switch, relaxed assertion, disabled concurrency or provider upgrade is
introduced. Non-Linux/local invocations directly execute the original command.

Blacksmith documents isolated per-job Firecracker microVMs in its
[security model](https://www.blacksmith.sh/security). This does not authorize a
shared kernel change: actual runner evidence must show KVM, a negative container
probe and Firecracker DMI identity before a private guest core pattern is changed.
The guest packet also retains boot ID, kernel, cgroups and CPU count. Container,
shared or unknown identity performs no kernel mutation and records core capture
as unavailable. A zero hard core limit also refuses capture. A changed private
pattern is restored after the original child; no shared host sysctl is modified.

The core ELF's actual Linux process PID (PRPSINFO, distinct from faulting thread
IDs) must match its `core.%p` filename and one retained producer start-tick
identity. Its actual executable map/inode, whole SHA-256 and physical x86_64 ELF
must remain identical before gdb receives that exact executable. Unknown,
unobserved, ambiguous or replaced producers refuse analysis; PATH Node is never
used to guess a core's producer.

Raw core files remain runner-local in a separate directory excluded from every
upload glob. Only a real core permits gdb installation and analysis. Public
packets contain its SHA-256/size and bounded diagnostics: registers, shared
libraries and backtraces with frame arguments and entry values disabled. No
locals, `bt full`, environment or argument strings are dumped. Each diagnostic
stream retains its whole SHA-256/size and explicit truncation status; at most
256 KiB of text is public per stream. Missing core/gdb is unavailable evidence,
never a successful crash diagnosis.

## Controlled original-source observation

The existing optional CI walltime workflow gains one independent job, admitted
only by the explicit `ci-native-fault-diagnostic` label event. Normal PR events,
main/merge-group events and all existing walltime inputs retain their existing
jobs and selectors. No unconditional extra Rust builder is introduced.

The job retains the instrumented orchestration/helper identity outside the
checkout before selecting the exact failed physical source above. It verifies
its head/tree/clean tracked source, installs that source's unchanged JS runtime
and invokes the original mandatory command once on the same 32-CPU runner,
warning options and Cargo parallelism. Thus old source does not accidentally
expect a future helper. Its failure stays a failure; no successful retry replaces
it. Source/provider identities and original native build receipts remain attached
as separate diagnostic evidence. Core availability and an actual fault frame
remain pending until this controlled Actions run completes.

## Validation and follow-up

Authored laws exercise the actual guest predicate with private, container,
unknown vendor and unavailable probe controls; execute the original child once
with unchanged argv/environment/streams and exit 0/7/SIGTERM/SIGKILL; and prove
unreadable diagnostic observations preserve an original exit 9. Workflow laws
retain original commands and verify the isolated explicit-label admission.
Receipts measure hashing bytes/time and each observation phase, including both
source snapshots and provider hashing. Command walltime can include observation
overhead; no unchanged timing or CI speed claim is made.

Exact-head Linux Actions, full protected suites, actual merge and causal crash
attribution remain pending. The #6830 p50/p90 target and full release gates are
not considered complete by this bounded diagnostic change.
