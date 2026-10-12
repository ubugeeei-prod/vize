# Same-run source CLI custody across failed-only retries

A failed-only GitHub retry can retain a successful producer from an earlier
attempt. The consumer must select that producer's exact artifact and retain its
original source/build identity while recording its own attempt separately.
This decision is paired with #6830 and the foundation CI qualification record.

## Actual failure

[Check run 38181774341](https://github.com/ubugeeei-prod/vize/actions/runs/38181774341)
at source `9cc08e0d94d0499ebd5978c9a491e13c0f0cacf2` has a successful
attempt-1 producer, job 114595790029, from 22:47:37–22:49:07 UTC on 2026-10-11.
Its complete original upload log names `source-cli-38181774341-1`, artifact
11718396194, ZIP 53,414,394 bytes, SHA256
`94a4d7469a43450ed4e970fd7966ce908ff33a87da8332f467354ca48350c92a`.
These are the original upload receipt fields; this investigation does not claim
independent ZIP extraction or a new Rust build of that artifact.

Attempt-2 consumer job 114599210123 retains archive SHA256
`8abbb5aa047a0be84d2b8c6124689739eb8e5f44fc8fcb0ff304497dfe830fcf`
and original producer receipt SHA256
`0e30772ff1be580fa3119c0fae8b725104d49039185cd9d2f4a260bf1757cb7a`.
It nevertheless requests `source-cli-38181774341-2` and fails before tests.
The attempt-2 jobs API re-exposes the successful producer under job
114599210411 with the same original start/end times; that is retained producer
history, not evidence of a fresh successful build in attempt 2.

## Contract

- Successful pack emits its producer attempt alongside its two existing
  digests. Reusable outputs retain that complete exact tuple.
- The consumer passes that tuple as one structured input. A common parser
  rejects unknown/missing fields, malformed digests, noncanonical/nonpositive
  attempts and a producer newer than the consumer. Download uses the same run
  and that exact producer attempt. No search, latest-artifact lookup or fallback.
- Original differential and producer envelope bytes remain unchanged. Restore
  checks actual raw Git HEAD/tree, current complete compiler/linker/config/flags,
  both original recipes, exact member order/modes/nonempty bodies, archive hash
  and original producer receipt hash before accepting the result.
- A separate `target/ci/vize.source-use.json` records schema/version, archive
  hash, original producer receipt hash, complete producer and consumer identities,
  both recipes and all three member rows. Its mode is 0644 with no path links.
  The verified use hash is emitted only after complete successful restoration.
- Cross-attempt SFC use requires that physical hash-bound receipt through the
  common verifier; ambient producer-attempt overrides do not authorize it.
  The existing whole loaded-oracle byte check remains. Normal same-attempt
  use keeps its original strict behavior and avoids compiler/whole-CLI rereads.
- Git reads for this custody use `--no-replace-objects`. A replacement namespace
  cannot turn a fake tuple into the authority for original physical source.
  An authentic original tuple remains valid with an unrelated replacement
  namespace present.

The `consumer-use` CLI mode gives Docker diagnostics the same common verifier,
using actual raw checkout and GitHub source/run/attempt plus the physical
original producer and use digests. It emits only the authenticated use digest
and both attempts. Missing, foreign, stale, changed, linked or nonregular use
custody fails without successful stdout. A mismatched attempt
without verified use custody remains an explicit diagnostic refusal. Backend
trust, sticky writes, compilation recipes, full consumers and their failures
stay unchanged. Complete receipt stdout is retained; log trimming is separate.

## Qualification

Real Git/tar transport laws cover producer 1 → consumer 2, original bytes/modes,
whole tuple/recipe/member mismatches, future/unauthorized attempts, missing or
changed use custody, inherited fixture environment and actual replacement
objects. A real subprocess law also proves the adapter's exact authenticated
stdout and its environment, receipt, mode, link, absence and argument refusals.
All 27 transport/workflow laws pass with zero skips, alongside scoped zero-warning
repository lint/format. The original 6d transport modules with a neutral inert-member driver
refuse only the 1/2 attempt mismatch; the candidate accepts that same operation
with original producer receipt and all three bodies unchanged. This proves
transport behavior, not a Rust product/compiler build or hosted acceptance.
Exact-head Actions, protected queue and actual merge remain required before
reporting delivery complete. The duplicate-build audit's broader optimization
candidates remain unimplemented and unmeasured.

The first source run 38186449336 successfully produces and restores the original
CLI, then fails three older workflow contract assertions that still expect the
former scalar digest input. Preserve that original failure. The successor
requires the exact complete retained producer input in all three contracts,
retaining upload/failure/tier checks and the single digest-strict download gate.
Those three failures reproduce locally before the change; all 41 related
transport and workflow laws pass afterward with zero skips. Production helpers,
actions and workflows remain byte-identical to the first source head.
