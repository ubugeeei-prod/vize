# Quoted nested dynamic arguments in selected SSR

Issue: [#8480](https://github.com/ubugeeei-prod/vize/issues/8480).

The original full Check run
[38035331735](https://github.com/ubugeeei-prod/vize/actions/runs/38035331735/job/114164470517)
reported four selected-versus-legacy SSR output differences. Two distinct
parent sources use valid `names]` and `events]` quoted keys followed by nested
index expressions. The lower directive classifier counted the quoted `]` as
syntax and truncated `state.keys['names]'][state.indices[state.index]]` to
`state.keys['names]'`. A new regression against the original classifier
fails with that exact argument while its seven existing tests pass.

The retained whole parent sources are 347 bytes (longhand) and 337 bytes
(shorthand), with the same original 1005-byte Child. Their lossless copies and
SHA-256 identities live in
`tests/_fixtures/differential/compiler/ssr-nested-dynamic-arguments/custody.json`.
The original mounted DOM/Vapor children completed successfully; their four
whole stdout packets were 9498 bytes with SHA-256
`3ad76a48e60a9595118ec9190333be1ed1cadc37803b9644fe6e64db50bc11a2`.
Those observations do not explain or close the older opaque fatal-child
failure in #7951. Its cause and actual engine remain unknown.
`before.full-ssr.log.txt` preserves the literal original four whole-module
selected/legacy RED records with their timestamps. That compiler baseline
has no actual SSR Node execution credit. Both new compiler routes print
their complete public results before assertions; the runtime input retains
their exact pair. When those entire results agree, executing the current
module executes the same emitted bytes as the explicit legacy result.

## Boundary decision

The lower classifier shares the existing total L1 lexical scanner through a
small borrowed-text API. The scanner's body, original lexer caller, quote,
escape, nested delimiter, template interpolation, and HTML recovery rules
remain unchanged. The API takes the text immediately after the authored `[`
and returns the original byte offset and closed flag. No second scanner,
pipeline stage, serialization, or legacy dependency is introduced.

For a complete argument, lower splits at the reported closing `]` and keeps
the original modifier splitting. For an incomplete name, it retains the
whole remaining authored text and no modifiers, including empty and missing
bracket controls. The shared v-pre and element-identity consumers retain
their complete selected/legacy result checks.

The existing fallible `VueDirectives::decompose` API is not substituted into
the total classifier. Its nesting/offset errors would require a separate
typed lower failure contract. This change neither hides such failures nor
adds an implicit legacy route. The scanner visibility change composes with
the separately owned adjacent-arrow change in #8471; no unmerged source
ancestry or frozen-head edit is used. Existing `=>` inside quoted attribute
names retains the current scanner's HTML-boundary recovery.

## Required evidence

- All three original complete SFCs must actually select `s4`.
- Selected and explicit legacy compilation must agree on every serialized
  public result field, including bindings, macro artifacts, diagnostics,
  CSS, whole code, and map.
- Source maps must be additive: disabling the map changes no other field.
- Both official and current whole map graphs must decode and round-trip,
  retain exact source content, and use valid generated/original UTF-16
  coordinates. Current maps retain their script-provenance-only scope;
  whole official template-map parity and argument anchor coverage are not
  claimed.
- Pinned Vue/compiler/server `3.6.0-rc.9` and plugin `6.0.7` must execute
  complete original Child and parent modules. Three selected-key states
  cover first, second, and reordered indices for both parent spellings.
  Twelve official/current renders must match independently written whole
  HTML expectations with no diagnostics or executed event handler.
- Raw runtime input, whole stdout/stderr, actual status/code/signal, and
  observed checkout source/tree/parents are retained before assertions.
  The complete packet is also printed in the original full feature recipe's
  raw job log. Actual Node metadata comes only from the executing helper.
  Checkout metadata does not identify the Rust executable or its build.

## Delivery

The original 31 mounted laws, compiler and canonical populations, SSR
10-case battery and 11 builtin controls, native gates, full recipes,
historical differences, and performance caps are unchanged. The n8n lane's
51-rule inventory is a rule identity witness, not 51 SSR compiler cases;
its full accuracy remains independently unfinished. No upstream change,
publisher edit, release graph change, or v0.441 inclusion is part of this
source repair.

Exact `2b192` full Check `38041695808` preserves zero-divergence original
SSR smoke results but fails to compile the new integration helper with
`E0583` at the crate-root module declaration. The helper moves byte-exactly
to `tests/nested_dynamic_arguments/mod.rs`, the ordinary crate-root lookup
location; fixture path depth, assertions, production, and recipes remain
unchanged. No new whole SSR runtime packet executed on that failed head.
Its separate Misskey server RSS result is 130.51 MiB over the unchanged
128 MiB ceiling; fresh full proof and the real LSP prerequisite remain.

Exact `ca0742` full Check `38045442545` executes all 24 complete compiler
packets, and the original-source selected/legacy result and additive map
checks pass. It preserves two failures: the shared v-pre/identity control
still differs on its original ancestry, and the actual Node child exits 1
with no signal before any of the twelve renders. The pinned official Vite
SSR module uses `__ssrInlineRender: true` and a `setup` function returning
its renderer, so the evaluator's separate `ssrRender` assertion rejects
that valid official Child. The evaluator validates the genuine shape per
route without invoking or replacing setup: official requires inline SSR
and setup; current still requires its separate SSR renderer. All authored
modules, complete map graphs, twelve HTML/state/diagnostic oracles and
shared lower controls remain unchanged. Fresh full execution on genuine
current main is still required; this failed run has zero completed renders.
A local replay of those exact source-built compiler modules and maps with
pinned packages reproduces exit 1 before the correction and exit 0 with all
twelve original outcomes after it. The local host is Node `26.11.1`, distinct
from the runner's `24.14.0`; this diagnostic replay is not new Actions or
current-main qualification. Earlier local missing-dependency failures stay
separate from the actual runner's renderer-shape failure.

TODO: obtain fresh exact-source full Actions, complete runtime/map packets,
unchanged performance and protected qualification, and actual merged main.
Local classifier failure and fixture custody do not replace those witnesses.

## Literal v-pre and normal-key prerequisites

[#8504](https://github.com/ubugeeei-prod/vize/issues/8504) records the separate
literal v-pre prerequisite exposed by the three original controls. Pinned
Vue retains literal Child/Other tags and `is` attributes. Before the opening
v-pre marker, the complete authored directive name is retained; after it,
and in descendants, only the longhand separator colon is removed. Quoted
brackets, outer argument delimiters and modifiers remain literal. The
original legacy output was wrong; it is retained as a historical result.

Both public lexer callback opt-ins default false. Only Armature opts into
lexical directive-head framing under inherited verbatim mode. Completed
name ends are temporary parser metadata, consumed once for each successfully
appended directive in opening order. Missing and extra custody are fatal;
normal directive raw_name stays the original prefix. The compiler alone
opts into an ordered frozen-element sidecar from the existing parser walk.
Its borrowed source must match both actual Root.source and caller source
before any Core mutation. External/default transforms carry an empty slice.
This is the original parser-bundle contract, not protection against forging
public mutable AST. Only frozen promotion and Vue-is removal are suppressed.
Newly literal, non-native component-like self-closing tags are accepted;
native, configured custom, unknown lowercase, strict and quirks policies
retain their existing boundaries. Public AST layout and default parse
result tuple remain unchanged.

The first scoped native debug measurement retained paired ordinal/end words:
CurrentElement 56→192 bytes, Parser 944→1104, and Core TransformContext
704→720, all aligned to 8. An end-only inline8 successor measures
CurrentElement 128 and Parser 1040; the context remains 720. It preserves
wide `usize` boundaries and uses the existing ordered append/consume walk,
without a rescan, narrow offset conversion or additional traversal.
Public ElementNode/DirectiveNode/RootNode sizes remain 104/176/224.
Across the same original five normal inputs repeated three times, both
inline8 designs add no allocation for 0, 1, 4 or 5 directives. Nine
directives add one spill allocation: 256 bytes initially, 128 bytes after
compaction, with no extra reallocation. Default and opted-in empty-sidecar
normal allocation snapshots agree; every complete normal Parser/Core AST
Debug result equals the retained untouched baseline. Internal size and
spill costs remain real. These native local debug measurements do not
establish a speed claim or replace the original 104 instruction ceilings.

Normal SSR nonidentifier component keys reuse the existing scope/binding
expression provider in both routes. DOM/Vapor, bare keys, local ownership,
globals and existing inline refusal are unchanged. The original Row first
failed at runtime with bare `keys`; after provider reuse, it instead
renders `data-key` while pinned Vue renders `datakey`. Both SSR component
dynamic v-bind consumers ignored `.camel`. They now use the existing
Camelize helper on `(key) || ""`, with one key evaluation and the unchanged
outer prop-entry guard. Events, static keys and options are not widened.
Separate finite controls check falsy keys, actual key/value call count,
spread order/collision, for/slot locals, helper imports, and raw low-level
maps. Scriptless public SFC maps remain null; nonempty low-level maps are
separate evidence, without whole official template-map parity claims.

The local source-built continuation passes the original three selected
controls, additive maps and twelve whole official/current SSR renders on
Node 26.11.1. Two new test expectations failed (default parser options and
an omitted existing falsy guard); those exact test laws are corrected.
The separate eight-case observer retains a genuine exit-1 `.camel` failure
after six renders and all sixteen official DOM/SSR modules. Its successor
collects every attempted runtime outcome before strict assertions. The old
six-outcome packet remains distinct. The source-built end-only continuation
completes all twelve original, sixteen whole-eight and twenty-four separate
camel observations with actual Node exit 0, no signal and whole raw streams
retained. Its complete runtime inputs and results equal the prior passing
paired-metadata source after only actual process PID removal; no fixture or
oracle normalization is needed. Four initially incorrect new falsy-key
expectations were corrected to the independently observed stock result
`[{}]`; their original failures remain retained. The original three/twelve/
thirty-one populations, authored inputs, historical packets and all caps
are preserved. These local Node 26.11.1 outcomes are not current-main,
Actions, release or completion evidence.

### Empty literal slot authority

The corrected frozen classification exposed the unchanged admitted
`<slot v-pre></slot>` case as an actual literal `ElementOp`, rather than a normal
`SlotOp`. The retained local camel continuation records the resulting genuine
`Legacy(Element)` failure. The original nonempty frozen-slot refusal remains a
separate admission boundary, despite stock Vue rendering that literal correctly.

The local successor passes the actual existing parser Root and its sealed
`FrozenElements` sidecar through the private SSR request. A borrowed view joins
both `Root.source` and the caller source before bridge work. The source bridge
then binds that view to the actual newly lowered L2 Region. Both selectors check
that exact source/Region before L3 lowering or codegen context construction.
The slot-only exception requires an authenticated opening at the full element's
start, its exact authored tag borrow and valid bounds, with no bindings or
children. Direct caller-built L2 has no authority; normal `SlotOp`, nonempty
literal slots and the other refused tags retain their prior routes. This trusts
the compiler's original parser/lowering bundle, not arbitrary public mutable
ASTs. It adds no scan, stage, serialization, dependency or metadata allocation.

Separate source controls were authored before this exception for mixed parser
roots/callers, equal-byte detached buffers, foreign lowered Regions, copied L2
slot offsets/tags and public DOM/Vapor/SSR literal-versus-outlet ownership.
They retain complete compiler results and raw maps; DOM/Vapor compiler pattern
checks do not establish runtime or stock source-map parity. The source-built
continuation passes the original admitted/refused/TypeScript/selector laws,
source/Region identity negatives and all sixteen complete public compiler
packets. A separate read-only local replay of the exact source-built public
SSR modules and pinned official modules completes eight observations over
four sources with equal whole HTML, context and diagnostics. Stock SSR maps
have empty mappings and current scriptless public maps are null. The original
slot refusal and both compile-repair failures remain retained. No DOM/Vapor
runtime parity or stock map equality is inferred.

The final end-only local session runs three locked/offline builds and
seventeen nonzero filters, including original native-lexer caller laws,
provider ownership, mixed-source rejection, missing/extra head custody,
literal/outlet/direct-L2 slot boundaries and the three whole runtime helpers.
All commands exit 0. Temporary allocator probes are restored byte-exactly.
The same four public slot controls additionally use the existing strict
completed-child collector for eight current/official observations in CI;
no original source or runtime population is replaced. All four supplemental
runtime directories join the existing always-run raw artifact retention.
Those latest test/helper changes await fresh Actions execution.
The first composed f44 Actions fails before full SSR execution on a checked-index
lint, observer formatting and a generated consumption row. Coverage also exposes
three supplemental modules as unintended standalone integration crates; fixture
paths are anchored before byte-identical ordinary module moves. The original
frozen parser options law identifies exactly two changed name-location endpoints
(23 to 15 and 28 to 16). Whole historical/current captures retain every original
input, error and expected digest, with all ten other ASTs byte-identical. The
public/default parser restores the original full-attribute metadata; only the
existing compiler opt-in uses the completed authored head. Exact old AST/error
controls bind that boundary without changing any of the twelve frozen oracles.
The f44 measured 100 stage plus four formatter rows pass all three repeats under
their original ceilings; that source-bound result does not qualify the successor.
The same four supplemental slot inputs now compare all sixteen complete public
compiler packets against the retained source-built modules, maps and diagnostics.
Preferred L0 imports retain their original grouped boundary, and only the two
affected compiler-consumer inventory shards are regenerated from actual sites.
The corrective 2e Actions preserves all twelve complete historical parser AST/error
records and passes the 104 three-repeat instruction measurements. Its current
native recipe exposes a missing passive-LSP test target from newer main. A single
ordinary composition with signed c10 main retains the complete incoming fixture
and recipe; the composed source requires fresh qualification without retry credit.
The ordinary current-main composition, complete source/full Actions,
unchanged historical corpora, all 104 measured instruction-budget rows,
protected queue and actual merge still remain before delivery. Raw previous
failures are evidence of their own source and never inherit this local pass.

### Default compatibility and source-bound lowering correction

The first exact source/full attempt exposed genuine legacy DOM and Patina
regressions: inherited `v-bind:x` became `v-bindx`, own `v-my:a.m` retained
`.m`, and literal classification changed default Component/Slot ownership.
Keep the original whole failures and corpus inputs. Restore ordinary Parser,
Patina, every public/default lower entry and empty Core context to the shipped
compatibility contract. Current Vue's own-after/inherited names differ; this
restoration is compatibility, not a claim of current official parity.

Complete directive heads now borrow the existing `raw_name` only for the
compiler's explicit frozen collection. Default parsing stores the original
prefix directly. The first successful `v-pre` position comes from completed
props, with no head vector or ordinal field. Armature still drops every
semantic `pre`; default level lowering keeps its original first-only policy.
The previous head-counter/storage2 source and allocation RED remain history.
No original-size or zero-allocation claim is made before new measurements.

One doc-hidden LEVEL entry, `lower_with_frozen_element_spans`, accepts only an
exact base-zero whole SourceBlock/tree and ordered borrowed opening spans.
All existing entries keep absent authority. Explicit empty custody is valid
only without frozen elements; missing, extra, duplicate, overlapping, foreign
or unmatched custody is an error before a Lowered escapes. The original
walk consumes spans at element dispatch, not lookahead, and validates leftovers
before materialization. It adds no scanner, pipeline pass or reverse legacy
dependency. The trusted SSR bundle first binds the parser carrier to both the
actual Root source and caller. Public mutable ASTs are not forge-proof inputs.

Parser and generic L1 HTML recovery differ for p/div, repeated button and
ignored nested form inputs. The explicit bridge refuses these precise custody
mismatches rather than inferring ancestry or dropping a supplied span. Default
recovery and all original malformed/facade streams retain their contract.

Keep the whole historical sixteen public slot packets. Eight SSR packets and
same four sources remain strict current/legacy/runtime controls. Six historical
literal DOM/Vapor packets describe the earlier regression. A separate eight-row
old/default baseline must come from the source-built authentic-ca observer,
including complete modules/maps/errors/options. The authentic-ca observer
now records all eight public results and nine whole default Parser/L1/L2
outcomes at ca074239ba (actual executable 599643aa, exit0, one test).
The fixed before data, exact observer source and source/recipe/raw-result
provenance live beside `public-default-ca.expected.json`; no corrective
current output supplies this oracle. All eight public maps are null. Fresh original layout/allocation,
104 measured instruction rows and 624 normal memory observations remain
required with unchanged inputs/providers/repeats/caps. The baseline's 57 exact
allocation-equality mismatches (40 up, 17 down) are a separate observation.

The closed-default source-built continuation completes seven locked/offline builds
and twenty-seven nonzero test commands, all exit 0. The original DOM six-case
suite, L2 v-pre control, Patina facade/pre/textarea laws and lexer caller laws
pass unchanged. Complete eight public default DOM/Vapor rows equal authentic-ca
except the declared observer route label; all nine whole default Parser/L1/L2
outcomes are exact. The four current child packets complete twelve original,
sixteen v-pre, twenty-four camel and eight slot SSR observations, with actual
Node exit 0, null signals and full input/stdout/stderr preserved before verdicts.
Temporary probes and every authored source byte are restored.

The same five normal sources repeated three times now measure CurrentElement
56 bytes and Parser 968, aligned to 8; Parser retains 24 bytes over the original
944-byte baseline. Core TransformContext remains 720 versus 704; current LEVEL
Cx measures 416 without an independently measured old Cx. Public AST sizes stay
104/176/224. All fifteen complete Parser allocation snapshots and five whole
ASTs equal the retained untouched baseline, including nine directives without
added allocation or bytes. Both default Core and opted-in empty-sidecar Core
have the same fifteen complete snapshots and five whole ASTs as that baseline.
These scoped local debug observations establish neither universal zero cost nor
fresh Actions, measured 104/624, protected admission or actual delivery. The
compile/feature-gate failures preceding this pass remain separate raw evidence.

Exact aa89 source Check 38078001865 exposes a missing storage-ledger row and
stale migration/v-on shards; full Check 38078037213 repeats the migration failure.
Register only finish.rs's two existing arena-Vec type paths with no allocation
category, and regenerate the two affected shards with the unchanged Node tools.
The reviewed diff adds two stage rows and the existing @b.stop/v-on:click.stop
spellings; source, fixture, policy and budget bytes are unchanged. Both generator
checks and twelve tests across the three existing Node source-law targets pass.
No Rust product build or Rust-wrapper suite ran. Original aa89 failures and
source-bound local/runtime/f44-104 proof remain retained; fresh source/full/native,
measured 104 and protected completion must qualify the corrected successor.
