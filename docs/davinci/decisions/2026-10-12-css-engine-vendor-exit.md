# Bounded CSS engine and archive dependency candidate

Existing issues: [#3864](https://github.com/ubugeeei-prod/vize/issues/3864)
and [#3295](https://github.com/ubugeeei-prod/vize/issues/3295).

## Decision and scope

Prepare two unofficial, publishable aliased forks using Vize's existing
`vize_oxc_parser` packaging convention. Keep LightningCSS alpha.72's complete
Rust implementation, original tests, feature surface and CSS `@import` bundler.
Apply only its two localized panic repairs and the three core parcel source-map
changes from `dbbd854f83c32fd8cb138489920e043c73da49c0`. This is a source
candidate, not an upstream release, delivered dependency migration or issue
closure. Fork publication and main-bound merge remain held for the API and
complete consumer evidence below. Root reviewed the preserved source provenance.

The isolated candidate starts on actual main
`a6633fb68b110d07abd5ee5a6049bc15f20f9c7f`, then replays onto actual main
`6d7aaacc45e9d9d9cb0ed65052c61fa93b5b04c5`. Its source-only import commit
`7199c4601c` contains unchanged selected source files. Packaging, migration,
regressions and this decision follow separately. The importer authenticates both
archives before creating either destination and refuses overwrite; each vendor
`UPSTREAM-IMPORT.json` records every selected original file's complete checksum.

## Primary source findings

The official LightningCSS [v1.33.0 release](https://github.com/parcel-bundler/lightningcss/releases/tag/v1.33.0)
corresponds to alpha.72 source
`c6a0c3cebf3395635e61075d2c81a96a710d4910`. Official current main
`ab14f90df45b1a94c6c10a3d9ca16b39d09f67e0` still declares alpha.72,
cssparser-color 0.5 and parcel_sourcemap 2.1.1. The later unreleased
`b0939ab684793d2ba3ccc1a91e0f4d3ba837a708` NonNegative restructuring
touches 33 files and is deliberately excluded.

Official cssparser-color main
`9f3ceddf398f5ce4ab76fdad1b42ccb6c578a926` declares 0.6 with cssparser
0.38, while the candidate preserves Vize's existing 0.5/0.37 provider identities.
The hue assertion remains present upstream and in this candidate. No complete
released hue repair was established. Registry API access returned HTTP403;
there is no claim about an unobserved newer crates.io release.

Official Parcel Source Map main
`b290dae1bea9bf249a0691e06014e09f974f70dd` and published 2.1.1 retain rkyv
0.7. Unmerged [PR130](https://github.com/parcel-bundler/source-map/pull/130)
at `b9d164f0db872fe139be6fdc00f99fdd767ccd97` has no qualifying CI checks.
Only its first [core migration commit](https://github.com/atezet/source-map/commit/dbbd854f83c32fd8cb138489920e043c73da49c0)
is imported. Its later borrowed-argument, edition, N-API, Wasm and unrelated
source changes are excluded. All upstream operations were read-only.

## Exact bounded behavior changes

`Percentage::parse` returns its ordinary `InvalidValue` error for calculation
trees that do not resolve to a percentage instead of executing `unreachable!()`.
The old `Calc::Value` success and ordinary percentage fallback are unchanged.
This does not claim newly correct evaluation of every valid CSS math function;
remaining guarded percentage over-rejections stay explicitly open.

HSL-to-sRGB retains the original f32 normalization whenever its result lies in
the required unit range, including signed zero and endpoints. Invalid finite
normalized results use `rem_euclid(360)`; non-finite hues normalize to zero,
following [CSS Color 4 hue syntax](https://drafts.csswg.org/css-color-4/#hue-syntax).
The cssparser-color assertion is not removed. The direct Rust census checks
65,536 actual f32 high-word inputs and compares full RGB/alpha bits with the
original conversion on healthy inputs; model arithmetic is not runtime proof.

Parcel serialization uses `rkyv::to_bytes`; deserialization uses checked
`rkyv::from_bytes` with rancor errors mapped to existing `BufferError`.
No unsafe `archived_root` remains in the migrated core. Existing public method
signatures and edition 2018 remain unchanged. Source/name/content, mapping,
offset, VLQ, JSON and data URL algorithms are not rewritten. rkyv 0.8 archive
bytes intentionally differ from 0.7; old archive compatibility is not promised
and the new tests do not execute the unsafe old decoder.

Current Rustfmt exposes three original alpha.72 formatting differences in
`bundler.rs`, `lib.rs` and `selector.rs`. A separate compatibility commit applies
only attribute/call wrapping and one blank-line deletion. Its exact successor
hashes are authenticated alongside the complete original import hashes. No
parser, selector, bundler or original test expression changes in that commit;
the existing `cargo fmt --all -- --check` command remains unchanged.

Root and the independent reviewer accepted the explicit vendor-policy follow-up:
only the two named imported packages join existing third-party exceptions to the
first-party workspace lint opt-in ratchet. LightningCSS uses the same
implementation-style Clippy policy as the existing Oxc fork and retains
`deny(missing_docs)`; Parcel retains its original `deny(clippy::all)`. The pending
first-party list remains empty and every first-party Clippy/format command and
contract remains unchanged. Current Clippy still has to execute on Actions.

## Packaging, licenses and public API impact

`vize_lightningcss` retains Rust library name `lightningcss`;
`vize_parcel_sourcemap` retains `parcel_sourcemap`. Both inherit the Vize
workspace version. Each consumer declares the package alias, local path and
exact registry version; Cargo's published manifest replaces the path with that
registry identity. A repository-only `[patch]` cannot establish published Rust
coverage and is not used for this migration.

LightningCSS keeps the complete original MPL-2.0 LICENSE and author/source
notices. Parcel 2.1.1 declares MIT and Jasper De Moor's original author metadata,
but its complete published archive and recorded official tree contain no
separate LICENSE file. The provenance record retains that fact without inventing
an upstream copyright notice. Root reviewed this exact archive/tree provenance
and preserved MPL/MIT/author metadata for the concrete candidate; no new human
approval flow is inferred from the absent file. No package has been published.

Renaming a package changes Rust nominal type identity despite retaining its
library spelling. In particular, the public
`vize_patina::style::ParsedStyleSheet::new` and `raw` methods accept and return
LightningCSS `StyleSheet` types; their typed rule visitors expose related
LightningCSS rule types. A downstream crate continuing to depend on original
`lightningcss` cannot assume that its values fit the aliased fork's API.

The focused Actions diagnostic executes the identical whole external consumer
against three dependency planes: old main plus original alpha.72; candidate plus
original alpha.72; candidate plus the aliased fork. It requires the first and
third to execute and captures the candidate/original E0308 nominal mismatch.
It also preserves the actual unwaived Patina cargo-semver-checks report and exit
code. A diagnostic that captures an incompatibility does not make that API
compatible. Root owns the pre-1.0 migration decision on concrete evidence.

## Preserved authorities and qualification

All 19 original CSS containment corpus files remain byte-exact, including the
99 native consumer observations and 90-process CLI authority. The complete
original `css_engine_panic_boundary.rs` body is also pinned in
`tests/_fixtures/differential/css-engine-vendor-exit/originals.json` for this
initial candidate. The 39-byte lch/sign, 66-byte number-fold and 1045-byte hue
fuzz artifacts are extracted verbatim into packaged direct engine fixtures with
fixed lengths/checksums. Existing healthy CSS, formatting and lint authorities
remain unchanged. No failed or old packet is silently recaptured.

The old boundary laws currently expect internal panic errors for some formerly
unguarded percentage cases and profile-dependent hue behavior. Those expectations
may intentionally fail after the engine repairs. Before admission, retain the
entire before authority and add an independently reviewed finite successor based
on complete source-bound debug/release emissions, without generic normalization,
dropping inputs or weakening healthy observations. This Draft remains off queue
while that work is incomplete.

The original source-map tests remain available. Four authored integration
contracts independently pin whole sources/names/content/mappings, literal VLQ,
JSON and data URL bytes, offset/composition vectors, and checked valid archive
roundtrips. A bounded child process requires four actual malformed archive
refusals (empty, malformed root, truncated tail and out-of-bounds vector pointer),
normal exit and exact completion markers. JSON features are explicitly enabled
in both assertion-enabled and release qualification.

## Delivery TODOs

- Run exact candidate Actions debug/release engine and all map/archive laws;
  retain full compiler/provider identity, raw outputs, test counts and failures.
- Review actual public nominal-type and unwaived SemVer reports, then record the
  maintainer's bounded pre-1.0 API migration choice before main-bound delivery.
- Preserve the reviewed source provenance/licensing and inspect normalized package contents; verify both
  aliases are published before their consumers when publication is authorized.
- Produce an actual isolated registry dependency graph with neither root patches
  nor workspace paths and no rkyv 0.7 route. Current local Cargo metadata and lock
  resolution are source graph evidence only and grant no published credit.
- Derive explicit finite successors for genuinely changed panic/hue results;
  preserve every original packet and all healthy byte-exact observations.
- Replay whole CSS `@import` recursion, media/supports/layer ordering, failure,
  cycle and CSS-module/source-map controls, original consumer/native/CLI corpus,
  and the complete existing protected suites with unchanged performance caps.
- Remove the audit waiver only with the candidate's complete no-0.7 graph and
  actual strict `cargo audit --deny warnings` success. Candidate source removes
  the waiver; neither issue is closed on source resolution alone.
- Keep current process-crash isolation until a separate evidence-backed
  maintainer decision. The numeric/math fuzz exclusions are already absent;
  the distinct #3926 unterminated-color timeout exclusion is outside this scope.
- Track actual merge and a fresh included release/installed CSS consumers.
  Upstream's release checkboxes remain unmet by an unofficial Vize fork.

No bundler, qualification item, instruction budget or first-party check command
is removed. No local heavyweight build, upstream write or public fork publishing
is authorized or performed by this candidate preparation.
