# Retain actual PR Rust build evidence

#9297's five cohorts selected the same37 package roots, but reported112–483 seconds
for the ci build. All emitted the same94 package/version labels and restored the
same sticky role and source-mtime counts. Package labels are not Rust unit
identities, and those logs do not explain the late LSP build tail.

Each existing PR cohort now forwards compiler-artifact JSON from its existing
Nextest build and requests Cargo's stable timing HTML. Package roots, features,
target selection, runtime filters, PR/MQ TSGO envelopes, Clippy, doctests, feature
lanes and job deadlines are unchanged. No build or test command is added.

The existing emitted-unit collector retains complete raw packets and only
fingerprint JSON authenticated by emitted filenames. Unhashed/absent/ambiguous
units remain explicit gaps. It copies no compiled objects or target archive.
Source/tree/compiler, the actual plan and selected runtime environment accompany
the packet. This does not prove CPU, linker or disk causality by itself.

A pre-build manifest records timing filenames, sizes and nanosecond mtimes. Only
the canonical cargo-timing.html alias is removed before the command. The after
manifest records every candidate and hashes copied new/changed HTML. Cached
unchanged reports and paths escaping the timing directory are excluded. A failed
or canceled build can have partial JSON or no fresh HTML; those gaps remain
visible in the always-uploaded evidence. The existing Node cohort runner owns
stdout capture and forwards the exact bytes to the console and file with stream
backpressure. Direct `exec` preserves Actions signal delivery; the runner awaits
file and console flush while its owned-process cancellation handlers remain
active. An output error drains that same Cargo group and fails the step. No tee
subprocess or waiting shell sits in front of the runner. Existing cancellation
controls and real mid-stream sink-failure controls retain unrelated processes.

Rust1.99.1 uses Cargo revision
[5f94df](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/util/command_prelude.rs#L498-L507):
bare `--timings` enables HTML without an unstable gate. Its
[writer](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/compiler/timings/mod.rs#L274-L283)
creates stamped HTML then links/copies the canonical alias. Pinned
[Nextest0.9.146](https://github.com/nextest-rs/nextest/blob/8af696ddcce8fff2962d6a5168b6d138b8616a35/cargo-nextest/src/dispatch/core/value_enums.rs#L381)
forwards JSON with `--cargo-message-format=json-render-diagnostics` while retaining
human diagnostics on stderr. The payload and collection cost require observation
in ordinary CI; no latency or zero-cost claim is made here. Common or split PR
archives are not introduced.
