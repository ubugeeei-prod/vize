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
ordinary canonical snapshot on the SSR-pug worker. The diagnostic authenticates
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

A diagnostic failure is collected without preventing the original full SSR
worker from running, then an explicit required step fails the job. Existing
SSR, pug, observer and aggregate failures remain fatal. Nothing is excluded,
deduplicated, cropped or retried through a fallback, and no production reader,
collector, oracle, accepted error or resource ceiling changes.

## Remaining work

Use fresh Linux evidence to distinguish root-prefix topology from actual
stat/open/read behavior. Only then choose and test an essential fix. Require
all original canonical observers on the complete input set before accepting
source readiness. This diagnostic grants no fix, product parity, P0 closure,
queue admission or release-publication credit. Do not publish a fresh release
candidate while deterministic complete-source blockers remain unresolved.
All upstream repositories remain read-only.
