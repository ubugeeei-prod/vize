# LSP exit evidence in full source coverage

The a00fb6c full source-coverage run [38929265536](https://github.com/ubugeeei-prod/vize/actions/runs/38929265536/job/117068745062) passed the original nested-loop rename/read vectors, then failed because its LSP process remained alive five seconds after `exit`. Vize had logged that it handled the notification and was stopping the transport. The log does not identify the remaining thread or wait; native stderr alone does not establish a cause.

A separate, temporary [instrumented probe](https://github.com/ubugeeei-prod/vize/actions/runs/38935630288/job/117090280899), based on exactly a00fb6c, passed nine selected cases: the unchanged original case, two normal-exit controls, and six passive-observation controls. Both LF and CRLF executions retained all 60 semantic request/result pairs, the real native TS2322/repair controls, shutdown response 62, exit code zero, and joined readers. Four complete packets passed their byte-count/hash manifests. No deadline occurred. This focused schedule does not reproduce, explain, or repair the original full-workspace failure.

The probe used Rust 1.99.1, cargo-llvm-cov 0.8.7, nextest 0.9.146, the full workspace feature graph/profile and eight test threads. Both native require/disable variables remained unset, as in the original coverage lane; the original test independently requires the native runtime. Compilation took 18m36s, tests 16.337s, and coverage export about 129s. The read-only source cache had no mtime manifest, so this is not warm-build evidence. The temporary workflow and successful-process timing files are excluded from this change.

Full Check [38988499789](https://github.com/ubugeeei-prod/vize/actions/runs/38988499789)
at `e2b883eed6378eb7ada73bf0220d89a185a3d6fd` passed source coverage:
18,532 tests passed, 44 were explicitly excluded, and line coverage was
85.5701%. Its native syntax artifact `12085125811` records the actual handoff
shutdown as `Clean`, with complete empty stderr; the child packet records
exit 0 and complete broken/repaired diagnostics. The original five-second
exit deadline did not fail. This broader non-reproduction still does not
identify the earlier hang's cause. The single case present in uninstrumented
Full but absent from coverage has an existing `cfg_attr(coverage, ignore)`
for unstable TS2589 output and passed uninstrumented; no new test exclusion
was added. The overall Full Check failed independent docs/tooling/performance
jobs, and subsequent combined source still requires qualification.

Full source coverage now opts into `VIZE_LSP_EXIT_FAILURE_EVIDENCE`, forwarded through the existing VP task environment contract. This enables the existing in-memory decoded-protocol ownership. Successful teardown does not build a terminal JSON packet or write files. Existing `VIZE_LSP_PASSIVE_EVIDENCE` consumers retain their previous full-packet behavior.

Only after the unchanged five-second exit deadline, before the existing kill, the helper writes the already-owned protocol and bounded, non-atomic Linux process/thread state and FD links. Reads retain raw bytes and explicit errors/truncation. Each packet keeps the existing 4 MiB cap and hash/completeness manifest. The deadline packet explicitly says stderr is unavailable until the unchanged reader reaches EOF; the terminal packet then retains all owned stderr after the original kill/reap and reader joins. Observation errors do not replace the original failure. The failure decision remains at five seconds; synchronous reads/writes have byte/count limits, not a wall-clock bound, so observation may delay the ensuing kill.

The existing source-coverage artifact also retains this failure directory. No test selection, vectors, timeout, kill/reap behavior, native policy, coverage command, or success gate changes. New Rust/config inputs and the declared environment invalidate the prior task-cache identity; final hosted qualification must show actual test execution, not a cached-success message. A recurrence can identify a blocked thread or descriptor; absence of a packet means no observed exit deadline, not proof of a lifecycle fix.

Focused controls cover no files on success, observation before the owner's kill, preserved deadline and terminal packets, write failures, explicit truncation, and lossless non-UTF-8 FD links. The existing VP execution test proves both capture variables reach the task only when declared and still rejects an undeclared variable. These helper/config checks do not substitute for the next full hosted source-coverage run.
