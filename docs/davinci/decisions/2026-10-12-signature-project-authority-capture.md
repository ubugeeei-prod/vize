# Passive capture of imported signature project authority

Issue: [#3952](https://github.com/ubugeeei-prod/vize/issues/3952).

The original `signature_help_tracks_imported_callable_edits` test passes on
actual main `d3b2f527d86b4d281207d58a23d05d6b92069d1a`, protected Check
38186830154 / Rust job 114611871296 (0.593 seconds). The independent Docs
candidate `4ae56daecf4cc31d2d2558b92451b07d448bc259` also passes the whole
original test in job 114616047549 (0.524 seconds).

The native config prefix's protected candidate
`4cc1ef9cbeb3ee33df3ba6086b174b611f8d5681` fails the unchanged second
signature expectation at tests.rs:211, Check 38187359884 / job 114613560751
(0.410 seconds). Its defaults child
`ff1c92144b9abf21f2ab306c2ccb342b7505a549` fails the same expectation in
38187360731 / 114613591130 (0.383 seconds). Both candidates were removed
from the protected queue. The older Docs `b3d8bceb` also contains this
config ancestry; its failure is not an independent main control.

Those failures retain complete original JUnit and stderr, including
`error handling method 'exit': EOF`, but do not retain signature-specific
native RPC bytes. EOF alone does not identify why the service returned
`None`: canonical document preparation, request failure and native null
remain distinct possibilities.

The proposed manual diagnostic reuses the original 4cc test archive:
artifact 11720466915, run 38187359884, ZIP SHA-256
`3a345f7b956970ebc0474f60d5ef9256463eedcaa46dd90cdd052b22c65c380f`.
It authenticates the entire ZIP and original schema-3 archive receipt,
checks out that literal source at its original baked workspace path,
and retains complete source, extracted metadata, original test binary and
the installed Linux x64 TypeScript 7.0.2 provider bodies and hashes.
Rust 1.99.0, nextest 0.9.146, full profile, required native runtime and the
original 30-second fixture timeout remain unchanged. It selects exactly
the existing test, adds no RPC, installs no wrapper and performs no rebuild.
The reviewed driver head is copied before the literal 4cc checkout. The
optional same-repository PR label allows one explicitly approved observation
before this new workflow reaches main; publication alone does not run it.

Archive extraction occurs before observation. Reusing its extracted
Cargo/binary metadata avoids tracing the five-gigabyte archive transfer.
Pinned nextest's archive loader remaps host/target Rust library directories;
manual metadata reuse otherwise keeps the original builder's sysroot. A
separate retained metadata copy derives only the original archive-aware
list's host and optional single-target libdir fields. Its whole inverse
restores the original object, and another untraced list must equal the
complete archive-aware list. With the same inherited environment, identical
RustBuildMeta and exact official libdir mapping select the same ordered
runtime library paths. Missing/multiple-target schemas or any changed list
fail before observation. The GNU ELF loader separately lists the actual test
binary's dependency paths, and their complete bytes and hashes are retained.
The same runner, inherited environment, linked-directory existence and
dependencies are checked again afterward. A mismatch never triggers a
fallback build or changed executable. The original metadata stays untouched.

Linux `strace` retains every followed process/thread's complete execution,
filesystem and read/write records. Native identity is joined to the
retained provider path, creation edges, terminal process records and actual
stdio pipe inodes. Complete kernel-confirmed buffers retain partial writes
and resumed calls. Abbreviation, missing child traces, unknown protocols,
ambiguous concurrent pipe ordering, framing failures and changed bodies
refuse capture completeness. The bounded parser supports observed LSP and
async API JSON-RPC; an unexpected MsgPack or socket transport is retained
as unsupported evidence, without completeness credit.

The receipt separates capture integrity, the original instrumented test
exit, actual responses and pipe consumption. Unread complete write suffixes
remain explicit. A missing second signature request is recorded without
sending a replacement. Whole raw logs and failure artifacts always remain
available. Instrumentation changes scheduling and timing, so neither a
successful capture nor an instrumented test pass qualifies production,
performance, the original full Rust suite, the whole 22-shard Matrix or a
release.

The same-root materialized-project transition remains a hypothesis. Before
a production change, prove the actual transition, physical imported source
identity and complete editor/API root state. Preserve unsaved authored
buffers and independent ownership; clearing them to the disk version is
not a valid repair. A source fix needs its own complete finite custody
successor and genuine original native qualification before queue admission.
