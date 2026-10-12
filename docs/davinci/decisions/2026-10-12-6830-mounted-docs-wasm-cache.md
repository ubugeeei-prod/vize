# Mounted Docs WASM target cache

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
This follows the independently delivered
[mounted Docs example cache decision](./2026-10-12-6830-mounted-docs-example-cache.md).

## Observed cost

The natural main Docs run
[38200947831](https://github.com/ubugeeei-prod/vize/actions/runs/38200947831)
on actual visual source `d0b62ef959e53f4224fdc76ce42a66cdcf6b19b6`
contains the accepted secondary-cache fix `0d63bc7e`.
Its original `build-playground` post step ran from 03:47:45 to
03:52:10 UTC on 2026-10-12. The whole retained log is 208,024 bytes,
SHA-256 `df4ba835bebf92fbb299491570284018821faa22b0e6bcaf7f283c7be6e843cb`.

The duplicate secondary Actions archive is absent, while its original
sticky provider commit succeeds in 3.063 seconds. The remaining primary
Actions archive is 14,800,610,247 bytes and takes 261.473 seconds to
archive/upload. The primary sticky provider commit separately succeeds
in 103 milliseconds. The whole post step is 265 seconds; these named
segments describe that source and run only.

This is evidence of duplicated storage work. It establishes neither
recurring provider eviction nor a matched end-to-end speed improvement.
The full Docs navigation, native, browser and public qualification remain
separate from cache finalization.

## Bounded change

Change three expressions in the existing Rust cache composite:

1. A trusted, physically mounted primary `docs-wasm` target registers
   no additional `actions/cache` save.
2. That case enters the existing restore-only action with `lookup-only`.
3. Its lookup uses the original mounted `target-seed-key`.

GitHub expression comparisons are case-insensitive. `Docs-Wasm` therefore
shares this eligibility, while the existing policy retains its actual
case-sensitive namespace values. `docs-wasm-extra`, `docs-napi` and
other roles retain their original eligibility and keys.

Every literal path, cache-version input, restore prefix and provider pin
remains unchanged. The policy, registry/git caches, secondary cache,
mount admission and sticky post implementation stay intact. The unchanged
provider post handles sync/unmount and commits only under its existing
successful-job policy. Setup, previous-step failure, cancellation or
ambiguous state can prevent a commit; RPC errors can warn. This change
adds no persistence or future-hit guarantee.

Mounted lookup never extracts an Actions archive over the sticky clone.
A trusted failed mount or GitHub-hosted runner retains the original
per-checkout restore/save path. PRs, merge groups, untrusted pushes and
non-default dispatches retain restore-only behavior. Compatible old
Actions seeds still restore through the same prefix. The selected mounted
role no longer refreshes its redundant Actions seed; a missing seed or
empty clone still runs the actual current-source builds.

## Consumer closure and controls

The complete source audit found 86 direct callers in 56 YAML bodies,
including eight original `./head/` action references, and four delegated
App E2E callers. The sole current `docs-wasm` consumer is
`build-docs.yml` / `build-playground`; both `docs-napi` callers are distinct.
The delegated producer emits the separate `app-e2e-*` namespace.
New ambiguous dynamic keys or Docs consumers require explicit review.

The source guard preserves the original ordinary roles and delegated
producer edges. It checks the whole dedicated WASM producer, its inherited
workflow environment and defaults, and physical recipe files and ancestors.
New direct, sub-checkout, delegated or case-alias WASM callers, changed
target/working-directory settings and ambiguous recipe sources are refused.
Ordinary-role growth and package-version changes remain eligible.

Local step actions must remain within the physically scanned `.github`
metadata, apart from the existing exact sub-checkout aliases. An outside
wrapper, traversal or symlinked metadata ancestor cannot hide a new cache
consumer. Reusable workflow jobs and the original memory installer remain.

All actual build commands remain unchanged: the independent generated
WASM cache controls only its original WASM condition; native npm packages
build current source; playground and Musea example builds still run.
The example check retains its dedicated `.docs-example-target` Cargo run.
No cache-hit output bypasses any of those commands or qualification.

The existing subprocess fixture reproduces a duplicate primary save
before the fix. The new law compares the entire original parsed action
against exactly three reversed expressions. It executes trusted event,
hit/miss/partial, case alias, failed mount, hosted, untrusted and other-role
branches with actual current-source artifact hashes and physical files.
An original stored seed plus a genuine later Git commit verifies stale
restore bytes cannot replace current-source work, and mounted lookup
cannot overwrite the clone or mutate the stored Actions seed.
Malformed input and a real primary subprocess exit 23 remain failures.

The original secondary-cache law retains its immutable pre-change JSON
hash and every fallback, stale-source, subprocess and Linux mount control.
It reconstructs the original three primary expressions before its
existing secondary rollback. Only legitimate mounted Docs primary-post
expectations change. The shared fixture remains byte-exact and models
neither successful sticky persistence nor failure-post commit outcomes.

## Delivery and measurement

The initial hosted lint run refused eighteen unawaited test registrations.
Explicit `void` registrations retain every callback body and all eighteen
controls; the complete original failure log remains recorded separately.

Focused source controls precede ordinary exact-head Actions, the original
full protected merge-group suites and actual merge. The actual Linux
mount law must execute on Actions; a macOS skip supplies no Linux proof.
There is no additional full-site campaign for this cache change.

Observe the next independently necessary natural Docs build after actual
integration. Confirm the selected primary archive is absent, original
lookup identity and provider post remain, and actual build status is
preserved. Report any measured post duration with its exact source and
run. All-CI speed, quota/eviction, future cache persistence and recurring
miss behavior remain unproved until their own evidence exists.
