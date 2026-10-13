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
`/home/runner/_work/vize/vize`,
and retains complete source, extracted metadata, original test binary and
the installed Linux x64 TypeScript 7.0.2 provider bodies and hashes.
Rust 1.99.0, nextest 0.9.146, full profile, required native runtime and the
original 30-second fixture timeout remain unchanged. It selects exactly
the existing test, adds no RPC, installs no wrapper and performs no rebuild.
The reviewed driver head is copied before the literal 4cc checkout. The
optional same-repository PR label allows one explicitly approved observation
before this new workflow reaches main; publication alone does not run it.
Only the exact newly added `signature-project-authority-capture` label triggers
the optional job; an unrelated label on an already labeled PR does not.

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

The shell's empty LD environment does not prove Cargo's child environment is
empty. The actual Cargo command, Rustup command, selected toolchain Cargo,
nextest commands and their complete bodies are bound before and after.
For a command whose whole bytes equal the actual Rustup shim, the reviewed
Linux Rustup policy prepends that selected toolchain's `lib` directory; a
direct selected Cargo command retains the empty shell LD environment.
Unknown command or Rustup policies refuse capture. Pinned nextest appends
this inherited suffix after its archived library paths, without deduplicating
the suffix. The GNU loader probe uses exactly that ordered value. Every
loader line must be a complete dependency or the explicitly recognized vDSO;
each reported path, canonical path and whole body hash is retained, and each
alias is resolved again afterward to reject retargeting.

The official [Rustup 1.28.2](https://github.com/rust-lang/rustup/blob/1.28.2/src/toolchain.rs#L182),
[1.29.0](https://github.com/rust-lang/rustup/blob/1.29.0/src/toolchain.rs#L184) and
[1.29.1](https://github.com/rust-lang/rustup/blob/1.29.1/src/toolchain.rs#L186)
have the same complete `set_ldpath` function (SHA-256
`1744c884c068b3f8d91c5357b37995b857fd6f56d592cbea09cdbbb26d1de9e2`).
Official [Cargo 0.100.0 external subcommand dispatch](https://github.com/rust-lang/cargo/blob/0.100.0/src/bin/cargo/main.rs#L305)
preserves that LD value; its package minor maps to Cargo 1.99.0 in
[the version implementation](https://github.com/rust-lang/cargo/blob/0.100.0/src/version.rs#L75).

Linux `strace` retains every followed process/thread's complete execution,
filesystem and read/write records. Native identity is joined to the
retained provider path, creation edges, terminal process records and actual
stdio pipe inodes. Complete kernel-confirmed buffers retain partial writes
and resumed calls. Abbreviation, missing child traces, unknown protocols,
ambiguous concurrent pipe ordering, framing failures and changed bodies
refuse capture completeness. The bounded parser supports observed LSP and
async API JSON-RPC; an unexpected MsgPack or socket transport is retained
as unsupported evidence, without completeness credit.
The selector includes explicit `fchdir`, which is a descriptor operation
outside strace's `file` group, alongside ordinary `chdir` in that group.
All retained processes belong to one observed creation closure. The selected
test execution must descend from the pinned nextest execution, and every
native execution must descend from that original test. Missing, reused or
foreign parent edges refuse credit. Cross-thread pipe operations that share
a timestamp or meet at an equal boundary also refuse unauthenticated order.

The installed platform package's complete manifest must be version 7.0.2
with Git head `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`. Its official
[API entry](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/cmd/tsgo/api.go#L20)
selects the async flag, the [server](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/api/server.go#L98)
constructs the JSON-RPC protocol, and the
[protocol implementation](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/api/protocol_jsonrpc.go#L9)
uses the same [Content-Length reader/writer](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/jsonrpc/baseproto.go)
framing as LSP. API initialization frames stay separate from the actual
editor LSP `textDocument/signatureHelp` requests; no API method is renamed.

The receipt separates capture integrity, the original instrumented test
exit, actual responses and pipe consumption. Unread complete write suffixes
remain explicit. A missing second signature request is recorded without
sending a replacement. Whole raw logs and failure artifacts always remain
available. Instrumentation changes scheduling and timing, so neither a
successful capture nor an instrumented test pass qualifies production,
performance, the original full Rust suite, the whole 22-shard Matrix or a
release.
Pinned nextest's human reporter writes to stderr. Both stdout and stderr
must exist and their complete bytes are hashed, but empty stdout is valid.
Fifteen focused parser, ownership, loader and refusal laws pass locally
without invoking the native provider or archived test. The actual optional
capture has not run at this source review; it receives its own later receipt.

The same-root materialized-project transition remains a hypothesis. Before
a production change, prove the actual transition, physical imported source
identity and complete editor/API root state. Preserve unsaved authored
buffers and independent ownership; clearing them to the disk version is
not a valid repair. A source fix needs its own complete finite custody
successor and genuine original native qualification before queue admission.

## Single retained observation and offline interpretation

The approved single label observation ran at driver
`0dba8eeb72f8a904b5b68de1c40b5599d5160889`, Actions run
[38193883416](https://github.com/ubugeeei-prod/vize/actions/runs/38193883416),
job 114632238453. The untouched archived case failed its original second
signature assertion at tests.rs:211:14 (0.359 instrumented seconds, exit
100). Its sole JUnit case has one failure and no skips. `context canceled`
in stderr is retained without treating it as a cause diagnosis.

Raw artifact 11722991118 is 91,991,916 bytes, ZIP SHA-256
`ea8c169ff88c2eb428bb05f4f9347a133d69baab5624aebde2709cce1a89d965`;
all 292 entries pass CRC and their whole bodies are retained. The original
integrity receipt is false: a Linux nested `/dev/urandom<char 1:9>` FD
annotation was not recognized. A separately reviewed offline decoder adds
only that finite nested character-device form, decodes exact `-xx` hex pipe
names while preserving every raw target, and distinguishes the two exact
nextest listing invocations from the sole exact-case execution. Neither
listing may own a native session. Unknown listing argv, duplicate cases,
failed listings, foreign native owners and invalid FD annotations refuse.
The original complete control assertions remain; its synthetic test exec
uses the same exact-case argv as the actual archive. Nineteen pure parser
and loader laws pass without executing any native process or test archive.

The first corrected offline interpretation still refuses completeness:
retained strace.7838 ends with `???( <unfinished ...>` followed by exit 0,
without a resumed syscall. That unknown observer boundary is preserved,
never dropped or promoted into complete evidence. Offline outputs are
separate from all 292 original files, the original driver, failed job,
JUnit and integrity receipt. No second observation or additional RPC was
performed. Partial source/root interpretation must name the missing
boundary; it cannot qualify production or erase the original failures.
