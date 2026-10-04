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
its external package. This is a source finding, not a new runtime reproduction.
The separate existing OXC module guard can already decline some syntax,
including uncertain U200B inputs; it remains unchanged.

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

Replay only this fifth repair onto that resolver in an isolated private
worktree. Preserve the producer's complete guard prefix, materialized-config
veto, original-source requirement, existing fallback, all 31 controls and its
paired receipts. The union adds the same five trivia controls for a total of
36 cases, requiring 144 complete ordered base/head diagnostic reports and
36 actual positive backend profiles. Earlier private f574 and 944 replay graphs
remain historical; neither they nor the accepted old `6edd` source packet
provide execution credit for the current union.

The three timed corpora, scripts outside the case matrix, plants, binary
custody, sample order, scheduler settings, timeouts and whole-report checks
stay exact. The 104 instruction benchmarks are a separate suite; all 100 level
ceilings and four formatter ceilings, methodology and ratchets remain unchanged.
Every current layer must pass exact-source Check, and the complete five-layer
union must pass contextual/API/CLI paired Actions, all 36 cases with 144 complete
ordered reports and 36 positive backend profiles, and fresh full-command
measurements before queue admission. All 104 instruction gates execute in the
protected merge queue and must pass there before actual merge; no separate
manual instruction campaign is required for admission. The global publication
freeze has lifted; this performance Stack remains separately held pending that
source qualification and owner approval. Actual protected validation and literal
merge remain required. Demonstrated 10x and native/default migration stay
unfinished.
