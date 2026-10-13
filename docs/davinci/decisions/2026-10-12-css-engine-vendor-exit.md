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
released hue repair was established. Registry API access returned HTTP 403;
there is no claim about an unobserved newer crates.io release.

Official Parcel Source Map main
`b290dae1bea9bf249a0691e06014e09f974f70dd` and published 2.1.1 retain rkyv
0.7. Unmerged [PR #130](https://github.com/parcel-bundler/source-map/pull/130)
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

The first exact-head run `38181994333` on `4e8e537734` genuinely failed its
new authored aggregate threshold in both profiles, after all 65,536 individual
RGB/alpha comparisons passed. The threshold incorrectly assumed more than 4000
finite fallbacks for this high-word domain. Independent std-only Rust recounts
in assertion-enabled and optimized builds establish exactly 61,660 healthy,
3874 finite fallbacks, two infinities and 254 NaNs (NaN resolves to missing zero).
The complete 262,144-byte little-endian input vector has SHA256
`9207d7eb28680a098c73dbe536d1ff7b94311dc417b9a385e0af6660683e93ca`.
The successor retains every row comparison and asserts those exact counts plus
one positive and one negative infinity. The original red logs remain retained;
this aggregate correction grants no additional product or published credit.

Parcel serialization uses `rkyv::to_bytes`; deserialization uses checked
`rkyv::from_bytes` with rancor errors mapped to existing `BufferError`.
No unsafe `archived_root` remains in the migrated core. Existing public method
signatures and edition 2018 remain unchanged. Source/name/content, mapping,
offset, VLQ, JSON and data URL algorithms are not rewritten. rkyv 0.8 archive
bytes intentionally differ from 0.7; old archive compatibility is not promised
and the new tests do not execute the unsafe old decoder.

Current Rustfmt exposes three original alpha.72 formatting differences in
`bundler.rs`, `lib.rs` and `selector.rs`. A separate compatibility commit applies
only attribute/call wrapping, one optional trailing comma in an existing test
call and one blank-line deletion. Its exact successor
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

Actual initial Clippy fails on 36 pre-existing Parcel source diagnostics under
Rust 1.99/Vize's configuration: root disallowed std String/format interfaces and
eight legacy spelling categories. Rewriting those public types or algorithms would
expand the bounded migration. A proposed, separately reviewed compatibility
follow-up retains the original deny-all and names only those actual diagnostics;
it is not an excuse to suppress first-party checks or change mapping algorithms.

The exact diagnostic census is disallowed_types: 22, needless_return: 4,
unnecessary_cast: 2, unnecessary_map_or: 2 and one each of the other six named
categories. `suspicious_to_owned` concerns the existing temporary Cow spelling;
its content is copied through `add_vlq_map`/`set_source_content` into owned strings.
The full original from_json algorithm remains unchanged; this is not a blanket
claim that suspicious diagnostics lack value elsewhere.

The initial strict audit also exposes original optional CLI's atty 0.2.14
(RUSTSEC-2024-0375/2021-0145) and proc-macro-error 1.0.4
(RUSTSEC-2024-0370), brought by Clap 3/clap_derive 3. No rkyv 0.7 route remains in the
candidate lock, but that fact does not make the audit green. A proposed one-line
optional Clap 4.5 manifest compatibility change must preserve the entire CLI source,
feature and original tests and pass their actual debug/release qualification.
No new advisory waiver is proposed. Initial JS/actionlint and dependency-direction
jobs additionally failed external download timeouts; their raw red evidence is
retained and new-source qualification remains required.

Root approved preparation of the separate finite named lint/optional CLI
compatibility change. The manifest retains `^4.5`; actual locked provider is
Clap 4.6.6, the same existing workspace provider. Resolving the change adds zero
packages and removes twelve obsolete Clap 3/atty/proc-macro-error route packages.
The manifest-only candidate preserved the whole CLI source and all 27 tests; both profiles
enable all features. Source review identifies a likely Clap 4 conflict-message
wording difference at `browserslist_targets_exclusive`; no original assertion is
silently rewritten or normalized. Actual failures must retain their complete
before packet and receive a narrowly reviewed compatibility solution.

Exact 514da all-features run 38184220273 compiles but CLI genuinely fails dev 24/27
and release 10/27. Clap 4 rejects the internal `output_file` group/argument ID
collision; the same `css_modules` collision yields release AnyValue type panics.
The narrow successor changes only four internal group/require references,
preserving all public flags, typed fields and output algorithms. Complete original
CLI source is frozen and hash-authenticated in packaged `cli-original.rs.txt`;
the group-only repair preserved all 27 test bodies. Both full red logs are retained.

The distinct target/browserslist negative in actual 514da release also fails only the
original capitalized `The argument` constraint against Clap 4's lowercase output.
Its finite successor changes that single literal byte and preserves the entire
old 27-test file in packaged `cli-original-tests.rs.txt`, authenticated against
the original import. Official Clap 4 formatter and the complete retained 514da
stderr corroborate the change; all other original expectations and the27-case
denominator remain intact. A separate authored contract requires full current
exit 2, empty stdout and literal complete stderr bytes. No generic normalization
or candidate-wide expected-output recapture substitutes for any original case.

Exact 9b112a focused Actions 38185552121 is terminal success on actual 9650ec6:
both profiles execute 121 library tests, all 27 CLI cases and the additional whole
conflict law, plus CSSOM/custom/browser and direct/map/archive contracts. All
three artifact ZIPs have authenticated API digests and retained complete bodies.
The nine downstream planes still contain three genuine nominal failures; the
analyzer's success cannot override them. Full old CSS laws remain unresolved,
and this focused result grants no product, healthy-corpus or publication credit.

The same source's normal JS format gate rejects eleven immutable upstream
metadata/manifests/config/test CSS files plus three owned TOMLs. Owned TOMLs are
formatted normally; five scoped format patterns protect only the authenticated
upstream originals/testdata, replacing destructive byte rewrites with the complete
custody checks and original functional tests. First-party files and other format
checks remain unchanged; the new exclusion still requires peer/Actions review.

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

A real package-only attempt on source18734 rejects explicit inclusion of the
reserved `Cargo.toml.orig` filename. Both repository originals remain byte-exact;
the packaging successor includes their exact copies as `UPSTREAM-Cargo.toml` and
lets Cargo generate its own `Cargo.toml.orig` for the effective fork manifest.
The custody guard compares both complete original/copy byte buffers. Packaging
verification still grants no compilation, registry or publication credit.

The repaired Parcel source package succeeds: fourteen files, complete original
manifest copied byte-exact, normalized active rkyv `=0.8.18` and unchanged MIT
metadata. LightningCSS's actual package attempt then correctly refuses because
the exact `vize_parcel_sourcemap` prerequisite is absent from crates.io. This
establishes the publication-order blocker, not a successful published graph.
The two forks must precede their Vize consumers; no registry patch or source
path qualification substitutes for that proof.

Renaming a package changes Rust nominal type identity despite retaining its
library spelling. In particular, the public
`vize_patina::style::ParsedStyleSheet::new` and `raw` methods accept and return
LightningCSS `StyleSheet` types; their typed rule visitors expose related
LightningCSS rule types. A downstream crate continuing to depend on original
`lightningcss` cannot assume that its values fit the aliased fork's API.

The focused Actions diagnostic retains its first complete new/raw consumer and
adds two authored whole controls for explicitly typed StyleRule/FontFaceRule
callbacks and an external CssRule implementation registered and called through
CssLinter. All three bodies run unchanged against three dependency planes: old
main plus original alpha.72; candidate plus original alpha.72; candidate plus the
aliased fork. Each first/third plane must execute; the candidate/original plane
must expose the respective E0308/E0631/E0053 nominal mismatches. Separate packets
prevent an early new/raw error from substituting for a trait/callback obligation.
It also preserves the actual unwaived Patina cargo-semver-checks report and exit
code. A diagnostic that captures an incompatibility does not make that API
compatible. Root owns the pre-1.0 migration decision on concrete evidence.

The first three-plane new/raw program actually executed on original main and the
fork, and the original-engine candidate produced the expected two E0308 errors.
Its first SemVer run failed earlier: the installed tool rejected Rust1.99's
rustdoc format v61. That is a tooling failure, not an API verdict. The successor
uses the existing release SemVer toolchain Rust 1.95, records its compiler/tool
separately from the Rust1.99 downstream execution, and preserves any actual
comparison nonzero exit. Initial artifact11718281627 and all three full
consumer/lock/output packets remain retained; complete nine-plane proof is new.

Exact successor18734 run38183205109 is terminal success on actual checkout
`fe76cc36ec1b619dc52eae45cc5c74475fd2ac12`, which contains the source commit
`18734dbe34c4c2542c13ea8bc3f4447e219ed119`. Independent authentication matches
all 82 retained files and nineteen relevant Git blobs. Both Rust1.99 profiles
execute all 65,536 RGB/alpha comparisons with exact counts, seven original/four
authored Parcel laws and four actual child refusals, plus two complete Lightning
map laws. Sole rkyv0.8.18 has std/bytecheck enabled. CLI/all-features qualification
was not part of this run and remains separate.

All nine whole downstream packets qualify: six baseline/fork executions exit 0;
candidate/original-engine reports new/raw two E0308, typed visitors one E0308 and
two E0631, and external trait one E0053, each exit101. Actual unwaived Rust 1.95
and cargo-semver-checks 0.47.0 report exit 0 with 222 passes/30 skips. That analyzer
result cannot override the three genuine nominal incompatibilities, and its
skipped analyses grant no coverage. Published/API compatibility and complete
normal/protected consumer delivery remain unproven; the PR stays Draft.

## Preserved authorities and qualification

All 19 original CSS containment corpus files remain byte-exact, including the
99 Rust consumer observations and 90-process CLI authority. The complete
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

An additional source-only observer executes all 99 Rust consumer observations (all 12
formatter, six style and 15 linter cases, each three times), all 66 inputs from the
eight original Atelier laws, and all 90 original CLI processes in both debug and
release profiles. A separate six-input vector preserves all five release-recovery
routes (including the 112-byte fuzz input) and the subsequent healthy source.
It retains complete results, source/compiler/graph/executable
identity and raw streams before analysis. Observer execution is diagnostic
evidence: it neither compares a new golden nor grants product, healthy-corpus,
API, registry or release qualification. Every original expected packet and active
boundary law stays frozen during this capture. Rust result fields are distinct
from N-API/public-service binding fields; no native-binding qualification is
claimed. Independently review changed
cells and verify unchanged healthy coordinates before proposing any successor.

The first observation campaign38187313931 on e6207b2 fails compilation in both
profiles (E0277): its CompactString digest comparison incorrectly used a JSON
Value rather than that exact string field. No consumer observation executed.
Retain the whole red packet; the narrow successor extracts the field's string
without changing any input, hash, expected authority or engine behavior.

The unchanged original `bundler::tests::test_bundle` genuinely fails in b2205cf
dev: authored `print`, then `screen` imports emit reversed media order. Parallel
load workers merge duplicate-path conditions under a mutex in arrival order.
Keep the whole red output and unchanged oracle. TODO: independently review
source-ordered condition folding in the existing graph/order phase while retaining
parallel loading, inherited conditions, source maps and every original import
case. Sorting names, normalizing output or retrying away the failure is insufficient.
The [bounded qualification plan](./2026-10-12-css-import-order-plan.md) records the exact preservation gaps and next before-capture slice.

Actual b2205cf [campaign 38187677652](https://github.com/ubugeeei-prod/vize/actions/runs/38187677652) is terminal failure solely for that dev
bundler test. Both observation jobs complete 99 Rust, 66+6 Atelier and 90 CLI rows
with independently authenticated ZIPs and complete profile parity. Rust has 33
known repaired rows/66 unchanged; CLI has 26 repaired rows/64 unchanged, with
no unexpected healthy drift. Old 66 laws have constraints, not invented whole-AST
before references. This remains diagnostic evidence; no expected authority changes.

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
- The concrete publication plan orders `vize_parcel_sourcemap`, then
  `vize_lightningcss`, before every consumer. Both new crate ownership and Trusted
  Publishing bootstrap remain unverified; configuration grants no registry or
  handoff credit and must be completed before an authorized publication.
- Exact 9b112a normal tooling reports a missing physical vendor manifest path in
  the publication adapter. Its bounded successor recognizes only the two named
  new vendor directories beside Oxc; existing full fake-publication tests still
  require every manifest and lockfile to be sanitized and restored byte-exactly.
  Peer review caught seven dotted development tables missed by both old guards.
  Quote-aware table classification and an independent fake-refusal predicate
  strip/refuse grouped and dotted root/target development tables while preserving
  complete normal/build sections, including a quoted target containing dots and
  the literal `dev-dependencies` spelling. Explicit whole-table laws run on Actions.
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
