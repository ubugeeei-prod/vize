# Required semantic completion and native terminal authority

Issue: [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

The frozen escaped-import case on the original `d55b8238` production source
returned the complete expected TS1260 and TS2304 vector, then failed the existing
empty-stderr gate. Retained runs 38143653116, 38145841289 and 38147452115 show
both an unconditional CLI-to-session degradation warning and the native
`error handling method 'exit': EOF` line. The original malformed input, program,
raw streams, verdict and benchmark budgets remain unchanged.

A parser abort needs a same-config semantic phase. The private CLI result now
separates that requirement from unexpected runtime degradation. Admission to
this category requires exit code 1, empty stderr and wholly recognized UTF-8
error-diagnostic lines; the existing source parser must also witness the syntax
abort. Exit 17, extra stdout, unexpected stderr and undecodable bytes remain
runtime failures. Their complete raw stdout and stderr are replayed, and their
original execution error stays unchanged. Ordinary semantic diagnostics and
clean CLI completion keep their existing result and diagnostic tuple.

The same configured native LSP process can return a successful shutdown
acknowledgement while its physical child subsequently exits abnormally. Generic
SDK custody therefore retains the actual owned PID, every stderr byte, terminal
status/signal, drain errors, timeout/kill flags and reaping result. The two
published Corsa 1.14.0 crates and their exact upstream-byte exceptions are bound
by [the receipt record](./2026-10-11-corsa-process-receipts.md). The SDK performs
no stderr interpretation or filtering and preserves default transports.

Canon constructs a separate observer for one editor LSP spawn. It classifies
only the entire 34-byte EOF line after a successful standard shutdown response,
a complete same-guard receipt, exit zero, complete stderr, successful reaping
and no timeout, kill or process/reader error. Additional bytes, NUL/non-UTF-8
bytes and unexpected messages are replayed in full, including any matching EOF
line. A nonzero or incomplete termination refuses successful batch completion.
Unexpected stderr with exit zero remains fully observable, so the original
strict stderr gate still rejects it. A deliberate discard has a separate
purpose and cannot classify the EOF line.

The per-call batch path explicitly closes the diagnosing client before accepting
its successful diagnostic result. If diagnosis has already failed, its entire
original refusal remains primary; a secondary close failure is retained in a
structured warning. Close errors are also observable when a destructor cannot
return them. This change does not authenticate API-only owners or establish
universal native teardown correctness.

Local evidence is capability and compilation evidence only. Genuine same-owner
configured LSP/API controls retain strict options, actual source names and the
full wrong/repair vectors; exit-zero EOF, exit-one context cancellation and a
real native watcher panic are distinct retained outcomes. Locked Rust 1.99
Canon library and test compilation passes using the existing provider-check
target. Pure packet controls require full-byte retention and refuse incomplete
process evidence. The combined real CLI/factory qualification, exact-head
Actions, original frozen 400 inputs and protected full suites remain required.
No measured speed improvement, public release or completed Stack is claimed.
