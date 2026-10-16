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
