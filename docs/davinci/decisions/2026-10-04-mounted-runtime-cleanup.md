# Mounted runtime cleanup and failure diagnostics

Issue: [#7729](https://github.com/ubugeeei-prod/vize/issues/7729).

Protected Check 37171788924 for the docs-only #7696 candidate `c7d3f8c7`
failed the controlled Transition law after a child-process failure. Its Rust
wrapper printed only empty stderr, concealing status, stdout and scenario.
One bounded unchanged-head rerun passed all 3,791 shard tests. The original
process status and failure cause remain unknown; neither a retry nor a
plausible unresolved top-level-await hypothesis establishes that cause.

Both mounted trace runners closed their happy-dom window in their own
`finally`, while the devtools helper closed it again after a failed trace.
Give the helper sole window-close ownership after setup or trace completion,
on success and failure. Runner finalizers still unmount and remove their
host. The helper closes once, then restores the observer; an earlier trace
callback rejection retains its exact thrown value even if helper cleanup also
fails. A cleanup failure after a successful trace remains fatal and still
restores the hook.

The Rust Transition wrapper reports the actual scenario and process status,
the complete original input Value, and full stdout/stderr as both readable text
and raw byte vectors on failure. The byte vectors retain invalid UTF-8 that a
readable conversion replaces. Input writes, process waits, success assertions,
JSON parsing, scenarios, complete goldens, source-built executions, native
custody and instruction ceilings are unchanged. No new pipeline stage or
product behavior is introduced.

Three deterministic laws cover successful completion, delayed undefined
rejection with failed cleanup, and cleanup failure after success. The six
existing devtools laws remain intact. All nine passed locally; actual
Transition and mounted runtime execution still require exact-head Actions and
full protected queue acceptance. This improves cleanup and diagnostics without
claiming the original intermittent process failure has been resolved.

Initial source `f0fe39c1` failed the zero-warning JS check because the mixed
async/synchronous cleanup array was awaited. Its disposer now explicitly returns
a Promise; cleanup order and all nine assertions stay unchanged, with no lint
exemption. Fresh exact-head acceptance remains required.

Incoming `fa614e5b` passed exact source Check 37173841416 after its genuine
asynchronous cleanup correction. Preserve that helper and its nine laws;
the additive whole-input and lossless byte diagnostics require their own fresh
exact-head Actions and protected acceptance. Original failed Check 37173428203
and its zero-warning job 111350962823 remain historical evidence, not a claim
about the corrected source or the unexplained original Transition failure.
