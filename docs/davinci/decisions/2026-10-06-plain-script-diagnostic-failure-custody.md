# Original plain-script diagnostic failure custody

Paired decision: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6016111536).
This public-input observation belongs to the existing diagnostic-empty
investigation. It does not establish a shared cause with the separate original
SFC CLI/editor parity failure, nor change release admission.

## Observed original failure

The official Rust worker 1 at
`e9046f46930d6524c8d70077c4c4118c2c1f045f` failed
`javascript_variants_keep_plain_script_ownership_with_misleading_vue_language_id`.
Only the complete `plain.cjs` version 1 and 3 publications differ: actual
`diagnostics: []` versus the original whole TS6133 hint for `text`, severity 4,
source `vize/types`, range `(1, 6)..(1, 10)`, and message
`'text' is declared but its value is never read.` Version 2 and the `.js` /
`.mjs` expectations otherwise match. The source filename, misleading `vue`
language ID, all original input bytes and every complete expectation remain.

The retained raw worker log is 976356 bytes with SHA256
`ecf18ea685b9efdd23cdf5cd2abee01b1d52c7967049716f18d8e0eea9c7bae1`;
the complete failed JUnit is also retained. The artifact contains no failing
plain-script process stderr or decoded protocol. The measured 33.842 seconds
cannot establish which acquisition route failed.

## Source-defined alternatives and limits

At the failed source, `DiagnosticService::collect` returns an empty sync set
for plain scripts. `native.rs:73-86` logs the existing outer 10-second timeout
without adding a diagnostic. If a bridge remains present, the subsequent
unavailable hint is suppressed. Non-timeout request/repeated bridge errors can
also become `Unavailable([])` in `corsa/collect.rs`; these are actual silence
routes, not proof that either caused this observation.

Successful native emptiness, non-Full report extraction, remapping/assembly,
configuration/program membership and source synchronization remain separate
possibilities. The authored config includes `.cjs` but omits `allowJs`; actual
native options and file/default-project membership were not retained, so this
does not diagnose the selected native program.

The pinned stock provider's source contract uses semantic
`textDocument/diagnostic` and emits Full diagnostics. Its readiness symbol
response is not substituted for diagnostics. This audit adds no query to
observe a project or report and grants no native-success credit.

## Minimal passive decision

The original driver opts into an in-memory record before its first existing
request. It retains every complete decoded sent/received JSON envelope and
publication. Only after the unchanged shutdown response, exit notification,
successful process termination and reader joins does a mismatching fixture
attach its whole buffered stderr bytes/readable text, terminal state, full
message record and original configuration to the existing mismatch report.
The record is decoded JSON, not raw wire frames. Other drivers allocate no
message record.

No production source, logging configuration, native probe/query, input,
expected vector, timeout, deadline, lifecycle ordering, SDK, budget or pipeline
stage changes. Existing INFO/WARN source, open/query counts and error/timeout
messages can establish the recorded path if the unchanged law fails again.
A successful mapped-empty count alone still cannot establish the raw native
Full payload, program/configuration or mapping cause.

Integrate this source-only change into the CLI owner's existing PR #8119
alongside its concrete completion fix; create no new issue, PR or draft.
Compilation and unchanged original-law hosted Actions remain required. Cause,
fresh whole protected qualification, actual merge and root-coordinated release
remain unfinished. Historical green cannot replace those facts; failed
unpublished #8115 is not resumed by this capture change.
