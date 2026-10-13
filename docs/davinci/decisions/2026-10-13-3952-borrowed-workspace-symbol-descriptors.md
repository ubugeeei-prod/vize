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

The existing warm-workflow changed-path admission names the private helper
and its test explicitly beside the workspace-symbol producer. It also names
the two Incremental descriptor/export paths for their shared borrowed entry. Its original
400-provider inputs, protocol, recipe, assertions and budgets are unchanged;
path admission alone grants no benchmark or current native acceptance.

The eliminated copies are established by the implementation. Their exact
contribution to the original peak is not isolated by source inspection.
The first repair head `eaac656d70827ea305cf8c8d27f2f097a564d262` passed
[the whole original Vue parity/churn job](https://github.com/ubugeeei-prod/vize/actions/runs/38335415414/job/115094274923).
Its sampled server peak was 128,168 KiB (125.164 MiB), an observed decrease
of 4,576 KiB (4.469 MiB), leaving 2.836 MiB beneath the unchanged 128 MiB
ceiling. The peak remained at `file-lifecycle-0`. Artifact verification
confirmed the complete fixture, runtime and budget objects match the original:
40 edit cycles, 20 file lifecycles, 287 publishes and 63 RSS samples. The
process-tree peak was 353,636 KiB, beneath 384 MiB with three processes.
The actual CLI source/recipe receipt binds that head and binary SHA-256
`68fa3e619887e77916e16a30cc8f796e6945c232aae8153ae2769269ec222e10`.
This is one source-specific measurement, not final-head or public acceptance.

Full qualification after the independent Nix transport repair, protected
queue and actual merge remain pending. Broader LSP correctness, the 40/63
resource targets and published-package acceptance remain unfinished.

The first exact-head source checks exposed stale generated consumer ledgers
and Clippy rejected the test-only `&Cow` argument. Regenerate both named
Maestro shards with their original Node generators and pass a cloned Cow
value to the ownership helper. All source-pointer, ownership and whole
descriptor/rejection assertions remain intact; production parser bytes are
unchanged by this correction. The initial failed checks remain failed.

The next source run retained the original Maestro direct-parser ceilings:
request paths zero and crate-wide 15. Its two failures exposed the initial
direct call in the new helper. Keep those assertions byte-exact. The borrowed
entry now lives at the existing Incremental descriptor boundary, sharing one
private parser/options function with the original owned `parse_outcome`. A
private Maestro alias delegates to that entry. Resident memoization, Arc
ownership and typed parser errors remain unchanged; no cache, parse pass or
request-path exemption is added. Revalidate the composed source on Actions.

The composed `4130df0` source passed the original whole Vue parity and Nix
gates. Its original400 comparison failed before measurement: the before
Incremental library compiled with `fresh: false`, while after reused the
identical `.rlib`/`.rmeta` filenames with `fresh: true`; Maestro then reported
E0432 for the new borrowed entry. The changed-dependency custody table lacked
Incremental. Add that exact crate/prefix to its existing clean-and-rebuild
selection and retain the existing linked-artifact `fresh: false` requirement
on both sides. New controls refuse stale, missing and lookalike dependencies.
Original400 inputs, release recipe, protocol, diagnostics and budgets remain
unchanged. The full failure artifact `11786336708` and source-specific successful
parity run remain separate from acceptance of the corrected source.

The distinct final `4130df0792ac0a5e7b39452b7d247d02ba1d822e`
[whole parity/churn job](https://github.com/ubugeeei-prod/vize/actions/runs/38339103331/job/115105861101)
measured 125,424 KiB (122.484 MiB), 7,320 KiB (7.148 MiB) below the original
failed sample and 5.516 MiB beneath the unchanged server ceiling. The complete
fixture, runtime and budget objects remained equal, with all 40 edits, 20
lifecycles, 287 publishes and 63 samples retained. The tree peak was 348,824 KiB
(340.648 MiB) with three processes. This is another source-specific observation;
it does not isolate the variance between repair heads or qualify the dependency
custody correction, final whole checks, merge or public release.
