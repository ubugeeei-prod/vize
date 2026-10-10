# Release confirmation terminal lifecycle

Issue: [#8436](https://github.com/ubugeeei-prod/vize/issues/8436).

The unchanged controlling-terminal release-abort law failed on two independent
source heads: #8381, Check 38029035846 / job 114145965341, and #8399, Check
38029026150 / job 114145927362. Both exited 124 after approximately 25 seconds;
the expected status is 1. Their output stops after the Vite+ command containing
`moon update && moon run ... release -- minor`, before the confirmation prompt.
Both jobs use Node 24.14.0, Ubuntu 24.04 and the pinned MoonBit compiler.

## Decision

Keep the actual controlling terminal, prompt-triggered answer, exit status,
`Aborted.` text, unchanged head/status/tags, 25-second PTY deadline and
30-second parent timeout. Capture bounded Linux process/session/foreground-group
metadata without arguments, input or environment values. Find the actual waiting
producer before choosing a repair. A green subsequent attempt does not explain
the failed attempts or complete this issue.

The diagnostic branch adds a manual-only job to the existing registered native
capture workflow. It preserves that workflow's original capture job. Its planned
cold and warm samples retain every failure; they are diagnostic samples, not a
retry policy or an acceptance exemption. The active 0.440 publication authority
and public manifests remain untouched.

## Evidence and remaining work

- Fresh-main macOS, Node 24.14.0 and pinned MoonBit: the original law passed in
  10.32 seconds with a cold native module, and 2.97 seconds after instrumentation.
  Neither result attributes the Linux failure to Node, MoonBit or the terminal.
- Diagnostic source 7a5105d258f8c8ab7af7401e01ff74b770d5f40f, Actions
  38030432750 / job 114150031838: the four planned original-law samples passed.
  Inspect their retained process evidence and reproduce the concurrent source
  tooling environment before deciding on a producer change.
- TODO: capture a failing Linux lifecycle, add a deterministic regression for its
  cause, repair the producer, and pass exact-head Actions and the protected merge
  queue. #8436 remains open until the repair actually merges.
