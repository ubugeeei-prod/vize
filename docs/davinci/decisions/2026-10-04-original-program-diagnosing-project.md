# Original Program diagnosing-project identity

Related: [#6849](https://github.com/ubugeeei-prod/vize/issues/6849),
[#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

The opt-in original Program adapter previously established authored root
membership and normalized options through the real Corsa configuration API,
then requested diagnostics from an independent LSP process. That process did
not attest its own selected configuration. A configuration appearing between
the front-end admission and LSP startup could therefore diagnose the original
source under a different project while retaining the root's admission metadata.

Move the existing native Vue project-info transport into a private shared
editor-session module in a move-only commit. Both opt-in adapters request
`custom/projectInfo` from the same process that returns their complete raw
diagnostics, before and after that report. The original Program adapter requires
the actual absolute configured path to equal its admitted root configuration
and remain unchanged. Missing, inferred, relative, foreign or changed identities
return `UnconfiguredSource`; communication failures keep their backend error.
Native Vue retains its existing `UnconfiguredProjection` semantics.
Unconditional session shutdown remains outside each request closure.

`OriginalProgramCheck::diagnostic_configuration_path` retains that actual
diagnosing-process identity alongside the same borrowed File, original
projection, URI/digest, complete raw report, normalized API metadata and every
authored range. Original root configuration bytes are retained outside the
worker closure and checked again before publication, matching the existing
final original-source check. Source and configuration materialization are
bypassed, as before.

The ten original JS/TS complete diagnostic vectors, including unused-variable
hints, remain unchanged and additionally assert the actual diagnosing config.
Real inherited strict-option reload and relative import laws remain intact.
New genuine negative process controls create a nested config only at LSP
startup or modify the root config bytes at that startup, then execute the
existing real checker. They require typed refusals, unchanged source and owned
process reaping. A small protocol-shape law rejects absent, inferred and relative
project receipts. It grants no positive backend authority.

This proves the selected diagnosing configuration identity and the bounded
root/source snapshot. It does not freeze inherited configurations, dependencies
or package metadata into one coherent graph revision. Broader native Vue typing,
JSX/TSX projection, complete fix-history comparison and default product
replacement remain unfinished. #6849 and #6879 stay open. New exact-source
Actions, protected full suites/all-100 and actual merge remain required for this
follow-on.

The preceding native Vue checker delivery is now terminal: #7554 actually
merged on 2026-10-03 at 15:41:21 UTC as signed
`5e3468fbfc15cde228619d357d70a97c6ac18e86`, contained in fresh main.
Its source `95ab5389cf41ad15b9568aba22e8b94cc9f28bfa` passed exact
Check37132327002. Protected Check37133244236, Musea37133243962 and
Nuxt37133243931 passed. All nine actual native Vue/backend cleanup laws ran
successfully again in the protected full suite, and all 100 instruction
benchmarks passed three identical measurements under unchanged ceilings.
The genuine JSDoc prerequisite #7598/#7599 had already actually merged as
signed ba766aaa/ae169129 at 14:53:31 UTC, after protected
Check37130191083/37130192076 and full/all-100 success. Historical Stack members
remain preserved; this new original Program correction starts independently
from literal main5e3468.

The earlier unchanged allocation-law failure76>74 and successful identical
archived retry remain recorded in the native Vue companion and #6830.
Process-global counter/libtest interference is an unproven hypothesis.
No allocation source or ceiling is changed here.

Initial source daa770467 Check37135594845 built the actual owned source and
passed all four Rust workers plus their source report. The genuine late-project
and root-config mutation/reaping law passed at .243s, all ten original full
vectors at 1.873s, inherited/import reload at .421s, and the original timeout
cleanup law at 1.210s. The tooling1 partition alone found the canonical consumer
inventory still naming the moved private file. Regenerate that inventory through
the same pure Node generator used by its Rust wrapper; its sole changed row
records the new path and actual import line. No local Cargo build, ceiling,
diagnostic vector or runtime source is changed by that correction. The refreshed
head still requires its own complete Actions and protected acceptance.
