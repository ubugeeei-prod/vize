# Retire retrospective template-emission reconstruction

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830), following the
[recorded historical debt](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6209033544).

## Evidence and decision

The scheduled [run 38970790588](https://github.com/ubugeeei-prod/vize/actions/runs/38970790588)
at `22a90c3f75b0b06a752d38fbf2c6591e32901f7d` passed the current twelve-fixture
TS-40 projection matrix, then failed historical reconstruction. Artifact
`12077326828` retains all 194 raw projection files. Its evidence archive SHA-256
is `9598dbbf1ad5d54f933f1cb994922dc6f42cb362e991d4252fe351fb0d677858`.
The authored Vue inputs and complete current output match their committed
identities; this is not evidence of a current projection regression.

The old inverse produces 36,507 Canon bytes with digest
`2e732969b949cf48aea6e9faa53a8219e735009665cae72c85505df95492e48f`,
against the immutable 34,478-byte digest
`a8d507b6cdcac5aa294febb15e2af30423660562b4cc82a9e689a839a5df2a56`.
Eleven generated helper changes from [#9122](https://github.com/ubugeeei-prod/vize/pull/9122)
account for exactly 2,029 bytes: library-generic classification and callable
member extraction from event-listener unions. [#9139](https://github.com/ubugeeei-prod/vize/pull/9139)
recorded the resulting current goldens. Reversing those reviewed changes in a
separate analysis restores the old full text and mapping vectors, but requires
another product-specific inverse and does not authenticate a historical
producer execution.

Retire this retrospective byte-equivalence proof instead of extending its
inverse after every intentional helper change. The historical proof remains
failed and is no longer a current qualification claim; it has not become a
passing receipt. No baseline, golden, compiler output or source mapping is
changed. This decision implements the requested reduction of costly historical
tooling while retaining product-critical validation.

## Preserved evidence and current qualification

The verifier is archived byte-for-byte as
`tests/_fixtures/differential/typechecker/template-dollar-emit/projection-receipt-historical.mjs.txt`
in a separate move-only commit. Its 11,746 bytes have SHA-256
`8041595eaffe1c17fdbda7cb39828895d44b67c9bf7ebb30f7aaab4d2a17d6cb`.
The frozen `projection-baseline.json` remains byte-identical with SHA-256
`e7fcc4e2239ce80fb0081b8fd075aa7bcf52339de5787e439b821dcdbb61e1b9`.
The six frozen source-recipe scripts and recorded source censuses remain
unchanged. Historical source replay continues to execute its authenticated old
source and whole original recipe; current code does not reinterpret that proof.

Only the current scheduled invocation and the four VM tests of its inverse
functions are removed. The current recipe is derived from the frozen predecessor
by deleting exactly that one command. Whole-recipe comparisons and source,
workflow, reference and permission rejection controls remain in force. Historical
timing entries and recorded source bodies remain evidence, not live invocations.

Current qualification retains the complete twelve-fixture Canon, content mapper
and Maestro packets, full text digests, mappings, semantic links, authored hits,
probes, diagnostics and drift rejection tests. It still captures all raw
projection files and executes the source CLI/editor product tests and both
pinned-native semantic receipt programs. Native phase measurements, budgets,
source custody, unknown-option checks and artifact packing are unchanged.

Focused recipe, source-custody, event and artifact tests validate this removal
locally. Exact-head Actions must qualify the integrated source; the old failure
and analysis are not substitutes for that execution or evidence of a performance
budget passing.
