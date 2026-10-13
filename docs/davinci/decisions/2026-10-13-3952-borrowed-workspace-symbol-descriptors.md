# Borrowed descriptors during workspace symbol search

Refs [#3952](https://github.com/ubugeeei-prod/vize/issues/3952).

The original 0.448 source qualification failed Misskey LSP churn at a sampled
server peak of 132,744 KiB (129.633 MiB), above the unchanged 128 MiB ceiling.
The exact source was `bd8755ef9df6b3678f08a4153a7cc9b69e315b8d`; the release
head was `2817a0aa13518dd46c16bc8bf56ac831f68c24dc`. See
[Check 38331701024](https://github.com/ubugeeei-prod/vize/actions/runs/38331701024/job/115082649528).

The peak occurred at `file-lifecycle-0`. The 40 preceding edit cycles peaked
at 109,472 KiB and the final lifecycle sampled 109,320 KiB. The lifecycle
creates and deletes one real SFC, checks its exact diagnostics and clears,
and searches workspace symbols to prove the index retains exactly one live
epoch. The original fixture has 583 Vue files and 888 Vue/TypeScript files,
at Misskey revision `810faa8e5db4bbf9d6d408a61df148559340ee99`.

Workspace symbol collection synchronously parsed each SFC through
`vize_incremental::parse_descriptor`. That one-shot function uses `parse_sfc`
and then `into_owned()` plus an Arc. The conversion copies the source and
template, script, style and custom-block contents even though the caller
already owns the source for the entire synchronous collection.

Use the same `parse_sfc`, filename and default options through a private
borrowed parser. The descriptor stays within that synchronous call; returned
symbols still own their names, URIs and ranges. Resident document memoization
and asynchronous request ownership are unchanged. Iterate script/style lines
directly rather than constructing unused line vectors. No query prefilter,
result exclusion, cache, extra pipeline stage or native-provider fallback is
introduced.

The regression laws inspect source/block ownership and compare complete
serialized descriptors and rejections with the original owned parser across
12 valid, malformed, escaped, Unicode, CRLF, external-block and Vue macro
inputs. Existing complete workspace-symbol and file-lifecycle oracles remain
unchanged. The original 128 MiB server, 384 MiB tree, three-process, cycle,
diagnostic, latency and timeout assertions retain scale 1.

The existing warm-workflow changed-path admission names the private parser
and its test explicitly beside the workspace-symbol producer. Its original
400-provider inputs, protocol, recipe, assertions and budgets are unchanged;
path admission alone grants no benchmark or current native acceptance.

The eliminated copies are established by the implementation. Their exact
contribution to the original sampled peak and the measured RSS recovery are
not established by source inspection. Fresh exact-head Actions must run the
whole original Vue parity/churn workload before this release blocker is
treated as fixed. Broader LSP correctness, the 40/63 resource targets and
published-package acceptance remain separate unfinished work.

The first exact-head source checks exposed stale generated consumer ledgers
and Clippy rejected the test-only `&Cow` argument. Regenerate both named
Maestro shards with their original Node generators and pass a cloned Cow
value to the ownership helper. All source-pointer, ownership and whole
descriptor/rejection assertions remain intact; production parser bytes are
unchanged by this correction. The initial failed checks remain failed.
