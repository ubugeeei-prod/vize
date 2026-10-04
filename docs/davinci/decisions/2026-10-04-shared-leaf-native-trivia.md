# Native module trivia in shared-leaf admission

Tracking: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698).
Prerequisite: [bounded shared leaves](./2026-10-04-check-shared-script-leaves.md).

## Finding and scope

Read-only review of Stack #7720 top
`401295c324635d0bd7205701daabc151453987e1` found that the original-source
visibility screen only ended `//` comments on LF. It used Rust's Unicode
White_Space predicate for trivia, which excludes FEFF and U200B. A source-bearing
non-Vue reexport such as `export * from` followed by one of those trivia forms
can therefore reach the existing `quote = None` continuation without examining
its external package. The initial source finding was subsequently reproduced
with five actual package-global failures, recorded below. The separate existing
OXC module guard can decline other uncertain syntax and remains unchanged.

The pinned native TypeScript 7.0.2 source at
[`2bd066d87f5bafd315be9f40889d0a60b9e58e0b`](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/stringutil/util.go)
defines LF, CR, U2028 and U2029 as line breaks and FEFF/U200B as single-line
whitespace. Its
[scanner](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/scanner/scanner.go)
actually consumes those characters. Retained source SHA-256 identities are
`439ae180861b0e9f11c000e8cdc68f8c0d5c6f47420904e81aaa626491b64a9c`
for stringutil and
`495f569872c21ce9c4ec6510a587f0249b3936a97f4500d4f6321f6fffb1e9d5`
for scanner, independently matched to that exact upstream commit.

## Repair and required evidence

Change only the existing trivia loop: consume FEFF/U200B alongside the retained
whitespace predicate, and end line comments on the four native line endings.
FEFF, U200B and NEL do not end line comments. No parser, graph walk, resolution,
pipeline stage, module owner or fallback route is added. Incoming backtick,
recovery, Vue importer-context and conservative operand guards remain exact.
The existing OXC admission parse remains the only module-metadata screen here.

Add five actual package-augmentation controls to the existing paired CLI matrix.
Each has a non-Vue `loader.ts` reexport separated by CR, U2028, U2029, FEFF or
U200B; only Comp0 imports that loader, while all four roots read the same cheap
leaf using the package's `LeafGlobal` type. The package specifier `leaf-types`
and loader contain no lowercase global/declare/namespace keyword that could
mask the trivia branch. The real package declares the augmentation. A new
partition law separately requires the original connected plan for those inputs.

## Genuine pre-repair runtime failures

An untimed local reproduction uses original baseline
`da66dc241cb7e6ad25fc52fbb4a1b2c1effec9e5` and preserved old producer
`f574ba5da9f0660dd8d53dd2c019befd42a8f35c`, with native TypeScript 7.0.2
on macOS 26.6.2 arm64. Rayon is one; outer servers are one or two. Native
`GOMAXPROCS` is cleared and `--checkers 1` is retained. Each case captures four
complete ordered reports and separate native profiles, for 20 of each.

| Trivia | Base, servers 1/2 | Old head, server 1 | Old head, servers 2                |
| ------ | ----------------- | ------------------ | ---------------------------------- |
| CR     | clean / clean     | clean              | `shared.ts:1:21 TS2304 LeafGlobal` |
| U2028  | clean / clean     | clean              | `shared.ts:1:21 TS2304 LeafGlobal` |
| U2029  | clean / clean     | clean              | `shared.ts:1:21 TS2304 LeafGlobal` |
| FEFF   | clean / clean     | clean              | `shared.ts:1:21 TS2304 LeafGlobal` |
| U200B  | clean / clean     | clean              | `shared.ts:1:21 TS2304 LeafGlobal` |

The exact diagnostic is `Cannot find name 'LeafGlobal'.` All three clean
reports in every case hash to
`fb6a6d287551c92821524c9c9c336e2ca699ee13c9359ad6d331bfc9b5c1d0c8`;
each failing two-server report hashes to
`5386de331f523a6e2d996c144da6f0f18aba9dda80db977c58a4b33d6cea75df`.
[The preserved data index](./results/typechecker-shared-leaf-native-trivia-old-probes.json)
links bounded JSON parts containing all five complete authored input sets and all 20 ordered reports,
the original row digest `bdd2e2350da59e21ba74c1ab2d9db6f500adff217b891fa98e1842a106c28073`,
controller digest and executable/native hashes, commands and profile manifest.
This local correctness reproduction establishes these failures only. It is
untimed and supplies no performance claim or execution credit for a successor.
The initial single-file archive at `ae1a8b44d0` exceeded the 350-line bound
and failed exact source Check. Storage now uses a 79-line digest index and
nine readable JSON parts, each below 350 lines. Reassembly preserves every
parsed source, report, command, profile and provenance object; no budget changes.

## Historical exact-source qualification

The fifth layer originally branched from resolver `401295c324` at exact source
`6edd86ed929b54366c6dbce8ff9b71ab399d5193`. Its [Check
37178381821](https://github.com/ubugeeei-prod/vize/actions/runs/37178381821) and
[paired run
37178381508](https://github.com/ubugeeei-prod/vize/actions/runs/37178381508)
finished successfully. Independent raw qualification checked 346 complete
commands, 26 cases with 104 complete ordered base/head reports across servers
1 and 2, and 26 separately captured backend profiles. All five new controls
matched the whole baseline report, with a positive single backend-command
count, no sharded span and zero truncation. Four retained split controls used
two actual backend commands; the other 22 cases used one.

The CLI comparison used original/main common ancestor
`da66dc241cb7e6ad25fc52fbb4a1b2c1effec9e5`, not the unsafe immediate parent.
The separate contextual/API comparison was parent-relative to `401295c324`;
all 12 complete fact shapes matched its 11 before/after pairs. CLI artifact
`11293713785` and API artifact `11294375691` retain complete raw processes,
inputs, reports, profiles, locks and source/build identities. These results
qualify that exact 26-case graph only. Corpus-specific timings do not prove
full-command 10x or general language soundness.

## Current five-layer composition

The shared-leaf producer subsequently advanced to
`59067a34ce54cb5e99cace756ee27a18f0fd452d`, whose production is byte-identical
to reviewed `944da15b00c406935ba5044df0d8b01cbeda5924`. It closes authored
source/compiler-option admission and retains 31 actual controls. The final
successor only types the JSX/extends fixture tuples and updates its receipt;
tuple values, order and callbacks remain unchanged. Resolver
`0bafe3b9e669b72a7deaba6b17bb3ab82f85d949` is a true descendant of that producer,
with resolver source/tests preserved from `401295c324`.

The fifth repair was replayed onto that resolver in an isolated private
worktree, preserving the producer's complete guard prefix, materialized-config
veto, original-source requirement, existing fallback, all 31 controls and its
paired receipts. The union adds the same five trivia controls for a total of
36 cases, with 144 complete ordered base/head diagnostic reports and
36 actual positive backend profiles. Earlier private f574 and 944 replay graphs
remain historical; neither they nor the accepted old `6edd` source packet
provide execution credit for the current union.

The three timed corpora, scripts outside the case matrix, plants, binary
custody, sample order, scheduler settings, timeouts and whole-report checks
stay exact. The 104 instruction benchmarks are a separate suite; all 100 level
ceilings and four formatter ceilings, methodology and ratchets remain unchanged.
Complete measured source `10995eeb3c3d7a54d76b3a8dfab3d9edb074a89f` passes
[exact Check 37180536528](https://github.com/ubugeeei-prod/vize/actions/runs/37180536528)
and [API/CLI 37180536363](https://github.com/ubugeeei-prod/vize/actions/runs/37180536363).
Independent raw audit verifies 396 commands, all 36 cases with 144 complete
ordered reports, 36 positive backend profiles and all planted diagnostics.
Each new trivia case uses one backend command without sharded work or truncation.
[The final resolver receipt](./2026-10-04-typechecker-snapshot-resolution.md)
retains all six paired full-command rows, cold samples and exact provenance.
Imported default max improves 345.461→328.316 ms and cheap-leaf default max
464.210→281.619 ms, each faster in nine pairs. Generated500 max remains
unresolved; these scoped synthetic results establish neither 42.55 ms nor 10x.
All 100+4 pinned ceiling policies and ratchets are verified; this is distinct
from actual instruction execution. A documentation-only successor must retain
every non-documentation blob and pass fresh exact-head Actions before admission. All 104 instruction gates execute in the
protected merge queue and must pass there before actual merge; no separate
manual instruction campaign is required for admission. The global publication
freeze has lifted; this performance Stack remains separately held pending final
head qualification and protected delivery. Actual protected validation and literal
merge remain required. Demonstrated 10x and native/default migration stay
unfinished.

## Queue staging conflict and replay

The qualified admission at final source `0a5b836f90` was removed after an actual
add/add projection conflict at #7719, not a failed diagnostic or timing claim.
Independent #7743 adds the evolved three-corpus helpers; #7719 still introduced
their two-corpus ancestors. Its projected bottom candidate `7489944cbb` and
original parent `7a4323281d` produced conflicts in the corpus and protocol files.
Final `0a5b836f90` and frozen #7743 `596183affa` already share identical corpus,
protocol and leaf-corpus blobs. [The paired issue receipt](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-5977489560) retains the exact source/projection facts and removal readback.

Dequeuing bottom #7702 removed all five entries atomically; no auto-merge
requests remained and independent entries were preserved. Bottom and top stay
draft holds until the reconstituted complete prefix qualifies. The isolated
replay inherits actual main `da894691fff4da81b4acbe71a935c7c67fb0539a` and all
existing meaningful source/test/fixture/documentation commits. The earliest
benchmark consumer now uses those final neutral helper bytes, hashes the leaf
corpus and honors its `--no-config` args. Production leaf admission and later
parity fixtures keep their original layer; final tuning and harness blobs match
the previously measured top exactly, while newer independent main fixes are
preserved. This is staging repair; it establishes no additional speed gain.

Fresh exact-head source Actions for every layer and complete 36-case ordered
diagnostic/profile proof plus controlled whole-command pairs against the new
actual-main baseline are required. Retain the old raw measurements above as
SHA-scoped evidence. Sequential projections with independent infrastructure
changes must be clean before native Stack admission; unchanged 100+4 ceilings
still require actual protected measurements. No prefix-only merge, actual
delivery or demonstrated 10x is claimed by this replay.
