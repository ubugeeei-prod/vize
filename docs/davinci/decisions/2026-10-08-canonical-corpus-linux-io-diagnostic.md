# Canonical corpus Linux source IO diagnostic

Refs [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) and
[#8262](https://github.com/ubugeeei-prod/vize/pull/8262).

## Observed failure

Check run `37738930679`, SSR-pug job `113185713196`, tested union commit
`b08d4899a6676651dbcf5d761e374311b3a16dd5` and source head
`3e1f18d5d0888d0cc89086e998dd2d49fbc01f0d`. Hydration and canonical identity
capture succeeded; the unchanged SSR source reader then failed with Linux
`ELOOP` while reading an UpNext.vue path containing 33 repetitions of
`packaging/deb/root`. Pug did not run after that failure.

The authenticated worker artifact `11533655886` contains the complete
44,367-path original Rust vector. Its native identity walk read every selected
source and observed its first rejected directory link at traversal 41. The
Jellyfin subset is 136 regular SFC blobs multiplied by all 41 aliases, giving
5,576 logical inputs. The canonical inventory contains 148 pinned gitlinks.
These complete inputs and alias multiplicity remain requirements.

At Jellyfin revision `6b35d977335cab224c4536d18dc22d15a63d5185`, the sole
committed symlink is `packaging/deb/root -> ../..`, blob
`c25bddb6dd4666c6eb8cc92e33f1d60f64c3162b`. UpNext.vue is a regular 3,748-byte
blob `f23ff4de9fa28533d163d8123dd78ca54622abef`; its committed ancestors
contain no other symlinks. macOS probes preserve the complete relative vector
and bytes across both root spellings but do not reproduce the Linux failure.
Root normalization is therefore an unproven hypothesis.

## Decision and executable evidence

Run `node tools/support/compat/github/canonical-corpus-io-probe.mjs` after the
unchanged original SSR/pug command, provided ordinary canonical snapshot
custody succeeded. This avoids additional diagnostic warming before the
original source reader. The diagnostic authenticates
the current checkout/run/tree, unchanged source collector, whole fixed snapshot
vector, all 148 gitlinks, Jellyfin pin and complete per-file snapshot digests.
It runs the exact extracted original Rust collector and tests every fixed input
under both the absolute corpus root and literal
`tests/davinci_test_support/../../tests/_fixtures/_git` spelling.

For each input it records Rust metadata, direct Linux `open64`, `File::open`
with complete byte reads, `fs::read_to_string`, and independent Node reads.
Successful Rust and Node bytes are compared directly, including the snapshot
SHA-256, length and Git blob identity. No path joining or normalization changes
the manifest spelling. Every unique full pathname prefix is checked with
`lstat`; symlink targets, original errno and failing operations are retained.
Both root variants must retain the whole ordered relative vector.

The diagnostic also materializes the exact pinned UpNext and symlink objects
in an isolated minimal workspace and repeats both root spellings without a
depth limit. This is a reproducer, not replacement corpus evidence. Raw Rust
stderr, all per-input operation records, prefix observations, vectors, toolchain
and generated harness identities are uploaded through the ordinary worker
artifact. The frame digest binds every successful Rust byte comparison.

A diagnostic failure is collected after the original full SSR worker, including
when that worker fails, then an explicit required step fails the job. Existing
SSR, pug, observer and aggregate failures remain fatal. Nothing is excluded,
deduplicated, cropped or retried through a fallback, and no production reader,
collector, oracle, accepted error or resource ceiling changes.

## First hosted result

Diagnostic source `43c01097e88e20a91d8591343604f884cd841188`, Check
`37744688355`, SSR-pug job `113203383993`, tested union
`f0ed020f0fca67f3f85372b92f1fd9c177b7c75c`. Artifact `11535835865` is
26,477,534 bytes, SHA-256
`29b72146568b707e6930117c4a2a3fe37e1ccfc8ec60653e7923db158fe43a02`;
all 53 ZIP member CRCs were independently verified.

Each full-root spelling retained all 44,367 ordered inputs and executed
177,468 actual Linux Rust operation frames. All four operations and every
independent Node read succeeded with complete source bytes matching the
snapshot. The two full prefix inventories contain 58,058 and 58,062 paths,
all successful. Both minimal-root spellings retained all 41 UpNext aliases.
The exact old 775-byte, 33-cycle failing pathname at zero-based vector index
13,636 also succeeded unchanged in the manifest-root probe.

The ordinary SSR and pug commands then passed with 44,367 inputs each; DOM,
production reach and complete aggregate worker custody also passed. The
ordinary current LSP worker at source `9fe63a3f20b7a56349bcca281e5f7fd98aca4adc`
also passed without the diagnostic. Its artifact `11535505346` retains the
byte-exact old whole vector, per-file byte digests, complete collector source
and Rust 1.99.0 toolchain. The actual old SSR worker had a target-cache miss,
no secondary/sticky target or nextest archive, and fresh compilation; no
cached SSR-binary explanation is demonstrated. Its test executable was not
retained, so source equality does not claim binary equality.

The old ELOOP remains unexplained. Neither root normalization nor a cache
repair is justified by these results. This first Check still failed the
diagnostic tests' four floating-Promise warnings under the unchanged zero
warning budget; explicit test registrations correct those warnings without
altering execution. Its diagnostic receipt also inherited the snapshot schema
label through object spread; correcting that diagnostic metadata and covering
serialized source custody leaves all IO records and counters intact. Fresh
complete source and protected delivery checks remain required. Preserve the
original failure alongside successful evidence.

The first attempt ran its diagnostic before the ordinary SSR command. Later
diagnostic revisions execute after that command, with an `always()` condition
and authenticated snapshot prerequisite, so the original source reader runs
without this additional pre-read. No historical success substitutes for fresh
qualification of the final order and receipt metadata.

## Remaining work

Use fresh Linux evidence to distinguish root-prefix topology from actual
stat/open/read behavior. Only then choose and test an essential fix. Require
all original canonical observers on the complete input set before accepting
source readiness. This diagnostic grants no fix, product parity, P0 closure,
queue admission or release-publication credit. Do not publish a fresh release
candidate while deterministic complete-source blockers remain unresolved.
All upstream repositories remain read-only.
