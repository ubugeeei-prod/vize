# Parser aborts require semantic completion

The native CLI stops after parser errors. Canon already completes diagnostics
through a session attached to the same generated configuration, so unrelated
semantic errors remain visible alongside syntax errors. That required phase
must not print the warning that the CLI runtime is unavailable.

Classify a parser packet as required completion only when its exit code is 1,
its stderr is empty, its stdout decodes as UTF-8, and every nonempty line is an
ordinary file diagnostic. Unexpected exit status, extra output and malformed
bytes retain runtime degradation reporting and the complete execution error.
If one shard requires completion while another has a runtime failure, the
runtime failure takes precedence. Ordinary semantic errors keep the CLI path.
An unavailable session still fails explicitly; a parser abort cannot become
an empty successful check.

The unit controls exercise normal and abnormal packets, shard precedence,
ordinary semantic diagnostics and clean runs. The existing native syntax and
semantic corpus continues to check exact mapped diagnostics before and after
a repair, plus denied-transport refusals. A subprocess control runs that same
corpus with fallback silencing disabled and checks the misleading warning is
absent. The reusable native Canon action runs the complete suite against its
existing shared Cargo build, frozen Vue dependency and pinned ElementPlus
corpus. Native execution is required and every captured packet is retained;
no full App E2E build is needed to qualify this handoff.

This is the typed handoff slice of PR #8675. Native process receipts, shutdown
lifecycle changes and the independent checked Corsa package graph remain in
that PR and require their own native and package acceptance. This slice does
not establish those outcomes.

## Observe the owner of the degradation signal

Full Check run `38929265536` at `a00fb6c` completed the exact native
syntax/semantic corpus, but its subprocess stderr contained `context canceled`
and the previously observed exit EOF notice. The outer EOF-only assertion was
the sole failure among 18,229 Rust tests. Corsa inherits native stderr, so that
stream cannot identify whether Vize reported a slower fallback.

The handoff test now installs a subscriber only in its isolated child process
and invokes the unchanged original corpus function. It requires zero events
from the exact `vize_canon::corsa::fallback` target. A subsequent real CLI
failure, with the actual native semantic session still available, must retain
the complete original diagnostic vector and emit exactly one event with all
owned fields: target, level, fallback, cause, error and message. The parent
also requires that control's exact human warning, preserving the once-per-run
warning slot after successful required completion. The child must run exactly
one test, pass, and report zero failed, ignored or measured tests.

A separate failure-only peer acknowledges API setup and then rejects the
actual project diagnostic request with cancellation (`-32800`) or an internal
error (`-32603`). Both must return the complete execution error, including the
original native parser refusal, rather than an empty successful result. The
peer never supplies successful diagnostic answers. Every raw child stderr
byte and complete refusal exchange remains available in the capture receipts;
stderr is also replayed to the test output without filtering.

The pinned TypeScript 7.0.2
[LSP server](https://github.com/microsoft/typescript-go/blob/typescript/v7.0.2/internal/lsp/server.go)
closes on the exit notification and can surface the dispatch group's
cancellation. Its
[CLI entry point](https://github.com/microsoft/typescript-go/blob/typescript/v7.0.2/cmd/tsgo/lsp.go)
prints a returned error to stderr. A direct native phase probe completed both
TS2322 and TS1128 diagnostic responses and the shutdown response before nine
retained traces printed cancellation during exit. Those processes exited 1;
this is evidence of the producer and timing, not proof of a clean lifecycle.
Other probe traces exposed real watcher panics and are retained as failures,
not accepted shutdown variants. Hosted unframed stderr alone cannot establish
the phase of that specific cancellation. This test changes no product error
handling and makes no claim that native shutdown is healthy.

## Reconcile the authenticated clean-shutdown law

Main #9271 now authenticates the native image and initialize response before
requiring Shutdown's null ACK followed by EOF. Its original two-Batch child
retains every diagnostic/stdout byte, all eight historical stderr/image
classification controls, and the final required `Clean` outcome. Historical
EOF warnings remain classified evidence and cannot satisfy that assertion.

The structured fallback signal runs in a separately named child test: required
semantic completion emits no fallback event, while the real degraded-runtime
control still emits the exact structured event and human warning. The
cancelled/failed semantic controls and their failure-only peer remain unchanged.
Both child stdout contracts now require one pass and six filtered tests, the
exact seven-test combined inventory. No native stderr is filtered or accepted
in place of main's clean-shutdown requirement. Fresh combined-source native
Actions remain required; this merge reconciliation is not runtime evidence.
