# Phase 1 — One Arena, Real Expressions

> [!NOTE]
> The highest-leverage phase: expressions parsed once, strings out of nodes,
> one lifetime. Exit requires a measurable performance **win**, not parity.
> Dependency chain is real here — order matters. In-phase fallback flags per
> charter #26; all deleted at P1-13.

## TODO index

- [x] P1-1 Arena unification in carton
- [x] P1-2 Allocator plumbing through armature/atelier
- [x] P1-3 `SourceLocation` diet: retire per-node `source` strings
- [x] P1-4 `SourceLocation` diet: retire dead line/column
- [x] P1-5 Retained expressions: parse-once storage
- [x] P1-6 Consumer migration wave A (croquis identifier/scope helpers)
- [ ] P1-7 Consumer migration wave B (atelier patch-flag, v-for, transforms)
- [ ] P1-8 Delete the fast/slow scanner split
- [ ] P1-9 Identifier prefixing as AST transform
- [ ] P1-10 Node strings → `&'a str` / arena atoms; delete manual `Drop`s
- [ ] P1-11 Arena reuse across files (batch pool)
- [ ] P1-12 Performance-doc truth pass
- [ ] P1-13 Phase exit: budgets pinned, old paths deleted

---

## P1-1 — Arena unification in carton

**Steps:**

- [x] `crates/vize_carton/src/allocator.rs`: `Allocator` becomes the unified per-compile handle `{ bump: Bump, oxc: oxc_allocator::Allocator }` — one value, one lifetime, one `reset()` covering both pools; `as_oxc()` exposes the retained-AST pool P1-5 parses into _(amended in-PR: the original step assumed oxc_allocator was still bumpalo-backed and the container aliases could flip here. At the pinned rev (0.142, `fc702c1`) oxc ships its own `Arena` — no bumpalo inside — and `oxc_allocator::{Box, Vec}` reject `Drop` payloads with a const assertion, which today's string-owning nodes trip. The aliases flip at P1-10 once the string diet makes nodes `Drop`-free; until then the two-pool handle is the chartered in-phase transitional state (#26))_
- [x] Audit bumpalo-API deltas used in-tree _(finding: `Deref<Target = Bump>` is preserved, so every `Box::new_in`/`Vec::new_in`/`BumpString`/`BumpVec` call site compiles unchanged; `alloc_str`/`as_bump`/`reset` keep their signatures; `allocated_bytes` now reports both pools' bytes — sole external consumer is vize_maestro's virtual-code metrics, where the sum is the truthful number; nothing needed a shim)_
- [x] Node-size static asserts from P0-9 unchanged _(no node type changes in this task)_

**Acceptance:** workspace compiles, full test suite green, P0 benches within noise, `tools/davinci/corpus-diff.mjs` empty.
**Deps:** P0-4, P0-9.

## P1-2 — Allocator plumbing

**Steps:**

- [x] Thread `&'a Allocator` through `vize_armature` entry points (`crates/vize_armature/src/parser/entry.rs`, `parser.rs`) and the atelier lanes (`vize_atelier_core::lane::transform`, sfc `compile_template`) so template structures and future oxc ASTs share `'a` _(landed: the six `entry.rs` parse wrappers take `&'a Allocator` and extract `allocator.as_bump()` at the boundary — `Parser::*` internals still see `&'a Bump`; `TransformContext.allocator`, its constructors, the four `lane.rs` transform functions, and the `lane/extensions.rs` wrappers carry the handle. The plumbing necessarily extends through the dom/ssr/vapor lane compile entries — they sit between sfc and the lane and a `&Bump` cannot produce a handle — and `compile_template_block` plus its vapor sibling lose their local `Bump::new()`: the birth site is one `vize_carton::Allocator::new()` per template block compile in `compile.rs`/`template_only.rs` (batch pooling stays P1-11). Deref coercion keeps every `Box::new_in`/`Vec::new_in`/codegen call site unchanged)_
- [x] `vize_atelier_jsx` (already oxc-based) switches its local allocator to the caller-provided one _(landed: `lower_source`/`lower_source_with_compat` gain a caller-provided `&oxc_allocator::Allocator` and the local `Allocator::default()` is deleted; the full-compile entries (`compile_jsx*`, `compile_to_vdom/ssr/vapor`) take `&vize_carton::Allocator` because they run the shared transform lane, and split it via `as_bump()`/`as_oxc()` at the lower boundary. Analysis-only callers (maestro, patina, canon) hold a carton handle and pass `handle.as_oxc()`; the babel pragma-probe's throwaway parse keeps its local oxc arena. Verified: workspace + all-targets + napi/wasm/legacy feature builds compile, both clippy gates clean, `VIZE_TEST_REQUIRE_TSGO=1 cargo test --workspace` 321 suites 0 failed; corpus-diff runs with the orchestrator's rebase, per protocol)_

**Acceptance:** corpus-diff empty; benches hold; no public-API regressions the CLI/vitrine can't absorb (charter #23: internal breakage is free).
**Deps:** P1-1.

## P1-3 — Retire per-node `source` strings

**Steps:**

- [x] `SourceLocation` (in `crates/vize_relief/src/relief/core.rs`): remove `source: String`; add `span: Span` (P0-9 type); every consumer from `davinci-road/plan/sourcelocation-inventory.md` switches to `Span::slice(source_text)` — the inventory, a generated artifact, regenerates to a zero-`source`-read scan as the migration record, and its generator now fails regeneration if a `.source` read comes back _(amended in-task: the original "inventory rows checked off in the PR" wording predates the file being generator-owned; the regenerated scan plus the generator ratchet is the checked-off form. Landed: `SourceLocation` is `{ start, end, span }` with the span derived from the offsets in `new`, kept in sync by `set_end` on the parser's text-merge path, and const-asserted at 32 bytes. The slicing basis is the exact string the parser parsed: `RootNode::source` in the template lane, threaded as `TransformContext::source`, a new `CodegenContext::source`, `SsrCodegenContext::source`, the vapor transform/generate contexts (`RootIRNode::source` now carries the real text instead of `""`), croquis `Drawer::template_source` / `VirtualTsGenerator::template_source`, patina's lint contexts and query-collector sinks, and canon's slot-outlet collection. JSX roots keep the root element's slice in `RootNode::source` for the source-map path while node spans index the whole module, so the JSX lanes pass the module text explicitly — `lane::transform_with_source_text`, a `source_text` override on the vdom codegen entry, and source params through the jsx ssr/vapor compiles)_
- [x] Diagnostic excerpt rendering reads via span + the file's source text (threaded where missing) _(landed: the only output paths that embedded a location's covered text were the debug-formatted error lists in the SFC template gate (`compile_template.rs`) and the vitrine binding parse-error strings; they now render through `CompilerErrorWithSource`, which prints byte-for-byte what the pre-span derived `Debug` printed — `source` field included, sliced from the span — pinned by an exact-equality test in `vize_relief`)_
- [x] Re-pin node-size static asserts (they shrink) _(P0-9 placed only the `Span == 8` assert, which is unchanged; the shrunk `SourceLocation` is now pinned at 32 bytes (was 48). Measured node sizes: ElementNode 208→176, AttributeNode 192→144, DirectiveNode 240→224, SimpleExpressionNode 144→128, CompoundExpressionNode 120→104, TextNode 72→56, InterpolationNode 64→48, CommentNode 80→64, ForNode 216→200, IfNode 80→64, RootNode 336→320, CompilerError 80→64)_

**Acceptance:** corpus byte parity **including diagnostic text**; alloc counts drop in P0-2 benches (record the delta); inventory file updated to all-migrated. _(P0-2 alloc deltas (`cargo bench --bench davinci -- --quick`, before → after): armature_parse large 479 → 300 (−179), stress-deep 517 → 395 (−122), stress-wide 321 → 220 (−101), medium 236 → 171 (−65), small 25 → 22 (−3), stress-interp 620 → 620 (±0 — that fixture's loc strings all fit CompactString's inline capacity, so they never heap-allocated); peak transient heap drops with them (large −11.3%, stress-wide −6.0%, stress-deep −5.6%). Tokenize lanes unchanged, as expected. croquis_analyze_* shows +1 alloc per run: the drawer now captures `RootNode::source` once as its slicing basis (`Drawer::template_source`), one template-sized copy that also appears as the peak-heap bump on the stress fixtures (stress-deep 6.4 KB → 16.2 KB) — the per-node string savings land on the parse side that feeds it.)_
**SemVer gate:** the removed `SourceLocation::source` and the threaded-basis signatures are public API on four published crates (`vize_relief`, `vize_atelier_core`, `vize_atelier_ssr`, `vize_atelier_vapor`), so the change carries a conventional breaking marker (`feat(davinci)!:` plus a `BREAKING CHANGE:` footer) — the declaration `docs/content/stability.md` requires of `cargo-semver-checks`. Charter #23 governs the waiver: Vize-internal crate APIs break freely until contracts GA in phase 6, while emitted output holds the hard byte-parity bar.
**Deps:** P1-2, P0-9.

## P1-4 — Retire dead line/column

**Steps:**

- [x] `Position` reduced to `offset: u32` (or `Position` deleted in favor of `Span`); line/col derived only in diagnostic rendering and `SourceMapBuilder::finish()` (`crates/vize_atelier_core/src/codegen/source_map.rs` — it already re-derives from offsets because parser line info was unreliable) _(landed as the **delete** option: with line/column gone an offset-only `Position` would have duplicated the span endpoints behind a sync invariant, so `SourceLocation` is now the 8-byte `{ span: Span }` (const-asserted; was 32) and every `loc.start.offset` / `loc.end.offset` read moved to `loc.span.start` / `loc.span.end` while P1-3's `loc.span.slice(...)` sites compiled unchanged. One byte-parity finding: the parser's `Position` values were degenerate by construction — `Parser::get_pos` binary-searched a newline table **nothing ever populated**, so every stored value was `line: 1, column: offset + 1` — and the sole output path that printed them (`CompilerErrorWithSource`, the SFC gate / binding-boundary debug rendering) has those frozen bytes pinned by check oracles (`tests/snapshots/check/vue-router-patch-oracle.ts` pins `offset: 157, line: 1, column: 158`). The renderer therefore reproduces the frozen `line: 1, column: offset + 1` shape from the span offset instead of deriving real values via `line_index`; flipping that output to true derivation is a recorded, corpus-visible behavior change for a future task to take through the corpus gate deliberately. Real-derivation consumers (source maps, patina, LSP collectors) were already offset-based and are untouched. Residual divergence window, unreachable in the test suite: v-for source sub-locations previously advanced line/column per-char over the alias prefix (`advance_position`, now pure offset arithmetic), so a **fatal** error on a v-for source expression whose alias prefix contains a newline or non-ASCII would have printed the advanced values where the frozen shape now prints — no test or oracle covers that combination, and offsets (what lint/LSP consume) are byte-identical throughout. Maestro's dead `source_location_to_range` / `internal_to_lsp_position` converters (0 callers) are deleted; jsx `SpanMapper` no longer builds a per-module `LineIndex` to eagerly expand positions. The sourcelocation-inventory generator now ratchets all five member paths and asserts the deleted carriers stay gone.)_
- [x] Delete the parser's vestigial line-tracking fields _(deleted: `Parser.newlines` (allocated but never pushed — the degeneracy above), `Parser::get_pos`, `Tokenizer.newlines` + `Tokenizer::get_pos` (production-dead: only its own tests called it) and the per-byte newline pushes in `Tokenizer::tokenize` / `fast_forward_to`; the text-merge path now calls `loc.set_end(offset)` directly. Node sizes (P1-3 → P1-4 bytes): SourceLocation 32→8, ElementNode 176→128, AttributeNode 144→72, DirectiveNode 224→200, SimpleExpressionNode 128→104, CompoundExpressionNode 104→80, TextNode 56→32, InterpolationNode 48→24, CommentNode 64→40, ForNode 200→176, IfNode 64→40, RootNode 320→296, CompilerError 64→40.)_

**Acceptance:** corpus parity including source maps; node sizes re-pinned smaller.
**Deps:** P1-3.

## P1-5 — Retained expressions: parse-once storage

**Steps:**

- [x] `crates/vize_relief/src/relief/expressions.rs`: `JsExpression<'a>` replaces the `raw: String + PhantomData` stub with `ast: &'a oxc_ast::ast::Expression<'a>` (+ `raw: &'a str` slice for display); decide parse point by bench — during template parse (armature) vs first semantic touch (croquis) — record the measurement in the PR _(landed as `{ ast: &'a Expression<'a>, raw: &'a str }` carried in the existing `SimpleExpressionNode::js_ast: Option<JsExpression>` — the `Option` is the honest boundary: `Some` iff the content parses as **one complete** TS-dialect expression covering the whole text (`SourceType::ts()`, matching the croquis consumers' `expr.ts`; completeness = whitespace-only remainder after `ast.span().end`, required because oxc `parse_expression` does not demand end-of-input and would silently return `a++` for `a++; b++`). v-for values (`item of items`), v-on statement bodies, and invalid text stay `None`; P1-6/7 consumers keep their own handling for those shapes. `raw` is the template-source slice when the accumulated content equals it (interpolations always; values without entities), else an oxc-arena copy of the decoded content (entity-decoded values, camelized same-name shorthand). **Parse point: template parse** — armature's four non-static build sites (directive value, dynamic argument, shorthand value, interpolation) call one `retain_expression_ast` helper (`crates/vize_armature/src/parser/expression.rs`); `Parser` constructors take `&'a vize_carton::Allocator` and split bump/oxc at the boundary. **One differential finding**: the unguarded retained parse crashed the #956 stack-overflow reproducer (`vize_atelier_dom`'s `expression_nesting_guard` suite — 2 704 nested `[`) because oxc's recursive parser cannot be depth-limited; the expression nesting guard therefore moved from `vize_atelier_core::transforms::transform_expression::nesting` to `vize_carton::expression_guard` (verbatim, import paths preserved by a re-export shim; six `scan` helpers widened `pub(crate)`→`pub` for the cross-crate comment-rewrite consumer) and the retained site refuses unsafe text (`expression_is_safe_to_parse`) **before** parsing and before counting — the same refusal every legacy site applies before creating its parse arena, so guard-refused expressions are parsed zero times everywhere and the counter law's "distinct expressions" means guard-accepted ones. Measurements, `cargo bench --bench davinci -- --quick`, before → after wall p50 (final tree, guard included): armature_parse small 1.53→1.76µs (+15.5%), medium 13.16→15.50µs (+17.8%), large 46.15→55.12µs (+19.4%), stress-deep 41.85→44.96µs (+7.4%), stress-wide 75.37→82.38µs (+9.3%), stress-interp 67.45→106.22µs (**+57.5%**, ≈78ns per expression incl. the guard scan; the pre-guard intermediate run measured +51.6%/≈70ns, so the guard's own share is small); allocs +1 per guarded expression with delimiters (the guard's scratch stacks, e.g. large 293→350) plus the oxc pool chunk in peak (+16–50 KB). Fused compiles pay the same absolute cost (dom stress-interp 159.6→196.6µs +23.2%, ssr large +22.1%, vapor large +17.3%, most others +2–12%; single `--quick` samples carry outliers — dom_compile_large printed +113.5%, inconsistent with its own +19.4% parse stage on a ~30% parse share, and the parse-free `atelier_ssr_codegen_*` stage drifted +22–25% on identical inputs in the same runs — the parse-stage numbers are the clean signal); croquis analyze windows exactly ±0.0% (parse outside the window). The P0-3 legacy probe counts are byte-identical to `expr-reparse-baseline.md` before and after (all 18 backend×fixture lines). The alternative, **parse-on-first-touch (lazy `OnceCell` per node + `retained_ast(&'a oxc Allocator)` accessor), was implemented and did not reach a benchable state**: any interior mutability over `'a` makes every template node invariant over the arena lifetime, and the tree's walk/visitor architecture is built on covariant shortening — 13 helper signatures de-unified mechanically (croquis virtual_ts ×2, atelier_core codegen ×4, atelier_ssr ×6 incl. arena-allocating `SsrCodegenContext`'s options, atelier_jsx ×3) still left 37 errors in vize_patina (incl. the public `Copy` rule-IR type `MarkupDocument<'a>` whose unified `&'a RootNode<'a>` underlies every markup rule) plus 43 more in maestro+canon and the jsx/patina test helpers: a workspace-wide variance refactor that would also tax every future consumer, permanently. Chosen by those numbers: template parse — its ~70ns/expression cost is transient for consumed expressions (P1-6/7 delete the legacy re-parses, today 0–500 per fixture/backend on top of this), while first-touch's cost is structural and forever. The template-parse regression above means the "benches hold or improve" clause is **not** met inside P1-5 alone; the phase design books the payback into P1-6/7 (their acceptance asserts the `davinci.expr.parses` drop and croquis bench improvement) — chartered transitional state (#26))_
- [x] Parse via `oxc_parser` with the shared arena from P1-2; template-expression parse errors keep today's diagnostic behavior (differential-checked) _(the single site parses into `allocator.as_oxc()` and **swallows failures** (stores `None`): today every template-expression diagnostic is produced by the legacy on-demand re-parse sites, which this task leaves running unchanged (`expr_parse_probe` intact, its per-fixture counts byte-identical to `expr-reparse-baseline.md` before and after), so diagnostic bytes are unchanged by construction — verified by the full suite + insta corpus passing untouched)_
- [x] Profiler counter `davinci.expr.parses` incremented at the single parse site; exported via P0-11 _(one `global_profiler().record_counter("davinci.expr.parses", 1)` per attempt inside `parse_retained` — an attempt is a parse, so the counter equals non-static expression nodes built by the parser whether or not oxc accepts the text; flows through the standard counter export (`--profile-json`). Law pinned by `crates/vize_armature/tests/davinci_expr_parses.rs`: a 9-distinct-expression template yields samples == total == 9 and `Some`/`None` exactly per the completeness contract)_
- [x] Lifetime note enforced: retained `&'a Expression` values are per-compile ephemera under the architecture's arena/cache contract — anything crossing a compile boundary (caches, folios, summaries) uses the owned serialized form, never the arena reference _(audited: `CroquisFolio` (vize_davinci) and `Croquis` (vize_croquis) carry no lifetime parameters — fully owned, so the compiler forbids storing the ref; maestro's resident virtual-doc ASTs keep the whole two-pool carton handle alive alongside the tree, so oxc-pool refs share exactly the bump refs' validity; relief nodes are not serde-serializable, so no serialization path can capture it. Node sizes: all unchanged (`JsExpression` 24→24 — `&ast` + `&str` replace the 24-byte `CompactString` stub with the `Option` niche preserved via the non-null ref; `SimpleExpressionNode` 104, `ElementNode` 128, `DirectiveNode` 200, `RootNode` 296 as at P1-4); the only node-size const assert (`SourceLocation == 8`) is untouched)_

**Acceptance:** the counter law matches the chosen policy — **each expression
parsed at most once (zero re-parses) always**; under parse-at-template-parse,
counter == distinct expressions; under parse-on-first-touch, counter ≤
distinct expressions and untouched expressions are provably never parsed.
Corpus parity; benches hold or improve.
**Deps:** P1-2.

## P1-6 — Consumer migration wave A (croquis)

**Steps:**

- [x] `crates/vize_croquis/src/drawer/helpers/identifiers/slow.rs` — reads the retained AST instead of re-parsing (its local `Allocator::default()` dies) _(landed: the node-aware entry `extract_identifiers_retained(expr, Option<&JsExpression>)` in `helpers/identifiers.rs` keeps the fast/slow dispatch byte-identical (heuristic factored into one `needs_ast_extraction`, shared with the legacy entries) and, on the slow branch, walks the node's retained `js_ast` through the same `walk_expr` (`extract_identifiers_retained_slow`) — for those nodes the local arena and its oxc parse die. The retained walk engages **only when `strip_js_comments` left the text unchanged**: the legacy slow path parses the *stripped* text and the stripper is not regex-aware (`/[/*]/.test(x)` gets mangled before the parse), so consuming the retained AST for rewritten text would silently *fix* that scanner bug — a corpus-visible behavior change that belongs to P1-8's waiver-reviewed differential, not this migration. On the comment-free path both sides consume identical bytes, making equality a parser-determinism fact. Fallback classes keeping the legacy re-parse verbatim until P1-8: `js_ast: None` shapes (v-for sub-expressions, v-on statement bodies, guard-refused or invalid text, compound expressions) and comment-carrying text (280 of the corpus's 37 540 slow-dispatch retained expressions — 0.7%). The single drawer consumer holding nodes is `check_expression_refs` (`template/ids.rs`), now feeding `SimpleExpressionNode::js_ast` through; its `ident_cache` stays keyed by content — the retained parse is a pure function of the same text (P1-5: `raw` string-equals content), so cache entries are path-agnostic. The refs variant `extract_identifier_refs_oxc` has no node-holding caller anywhere in croquis (canon/maestro pass owned croquis strings, which can never carry arena refs by the P1-5 lifetime contract) — untouched, collapses at P1-8)_
- [x] `crates/vize_croquis/src/drawer/helpers/v_for/oxc.rs`, `visit_element/second_pass.rs` — same _(second_pass landed: `<component :is>` reference recognition (`component_reference_expression`) shape-checks the retained AST directly and the legacy throwaway parse dies for `Some` nodes; `None`/compound keep it. v_for/oxc.rs is a **recorded no-fork**: its parses are binding-pattern parses over synthesized `let [<alias>] = x` text — text that never existed as a template expression, so no retained AST corresponds to it — and the v-for value's own retained AST must not be consumed either (JS `in` associates left while Vue's v-for grammar splits at the first viable `in`/`of` boundary; they disagree on `a in b in c`). The drawer also holds no per-compile allocator handle (`Croquis`/`Drawer` are owned and lifetime-free under the arena/cache contract), so per the fallback clause the local arenas stay for exactly these binding-pattern parses, recorded in the module doc; they unify at P1-8)_
- [x] Differential lane: for one release, CI compares old-path vs new-path results (identifier sets, v-for destructure shapes) over the corpus; divergences are bugs in one side — investigate, don't average _(landed as `#[cfg(any(test, feature = "davinci-differential"))]` dual-run comparators **inside the migrated helpers** — every retained identifier walk and component-reference check re-runs the legacy re-parse and `assert_eq!`s (divergence panics; nothing averaged) — plus process-global counters in `vize_croquis::drawer::differential`; zero-cost when off (module and call sites compile out). The plain suite arms them via `cfg(test)` (`drawer/tests/differential.rs` pins a coverage floor so a cfg regression that disarms the lane fails loudly). Corpus-runnable entry, exact command: `VIZE_DAVINCI_DIFFERENTIAL_CORPUS=tests/_fixtures/_git cargo test -p vize_croquis --features davinci-differential --test davinci_differential -- --nocapture` (the committed battery + P0-2 ladder run with exact-pinned counts — 8/3/2 battery, 24/5/4 with ladder, the pin doubling as a dispatch/retention ratchet — and the env var widens the sweep to every `.vue` under the dir). Measured 2026-08-15 over the hydrated corpus: 41 580 files analyzed, **37 260 identifier comparisons and 1 649 component-reference comparisons, zero divergence**; 13 235 v-for shapes witnessed (a pre-gate sweep that also dual-ran comment-carrying text measured 37 540 identifier comparisons, likewise zero divergence — the comment-free gate is a proof-shape decision, not a reaction to a found divergence). The v-for lane is a witness count, not a dual-run: wave A forks no v-for path (previous step), so old and new are the same code and the fork-level comparison becomes real with P1-7/P1-8; v-for shape parity over the corpus is carried by the standard output-level corpus gate meanwhile)_
- [x] Charter #37 note: only the _input layer_ moves; tracker internals untouched _(holds: the diff touches the identifier/component-reference input helpers, the `ids.rs` node plumbing, the v_for no-fork record, and the cfg-gated lane; `scope.rs`, the trackers, and every croquis product type are untouched — `node tools/davinci/croquis-consumers.mjs --check` reports the consumption matrix unchanged)_

**Acceptance:** differential lane green over the corpus; `davinci.expr.parses` drops (record); croquis bench improves. _(Differential: green, counts above. Counter record — the plan line oversimplified: `davinci.expr.parses` counts armature's retained parses and is **unchanged by construction** (wave A deletes croquis-side re-parses, which were never in that counter, nor in `expr-reparse-baseline.md` — that baseline instruments atelier only and its 18 backend×fixture lines were re-verified byte-identical before and after this task from the bench stderr). What drops is croquis's uncounted analysis-side re-parse total: each identifier/component-reference comparison above is exactly one avoided `Allocator::default()` + oxc parse in production — 38 909 avoided parses per corpus analysis sweep, 18 per full+for_compile ladder sweep. Bench record (`cargo bench --bench davinci -- --quick`, P1-5 tree → this tree): croquis_analyze_full_large p50 177.4→165.6µs (**−6.7%**; a pre-gate pass read −11.6% — single `--quick` samples), allocs 615→592 (**−23**, deterministic in every run); for_compile_large allocs 297→296 (the `:is` parse; its +3.4% wall print is single-sample noise against the alloc drop); remaining croquis lanes −6.8…+2.0% wall with allocs unchanged (few complex expressions). Armature and fused lanes: alloc counts byte-identical on all 66 lanes; wall within `--quick` noise (armature −4.3…+2.5%; dom's first after-pass printed +49…+107% outliers on alloc-identical lanes and a rerun read −10.2…+3.4% — the same single-sample outlier mode the P1-5 record documents; ssr −3.1…+3.9%; vapor −7.8…+11.4% on sub-µs lanes). The P1-5 parse-side regression itself is untouched here — its remaining payback is the atelier re-parse deletion, booked at P1-7 as the phase design states)_
**Deps:** P1-5.

## P1-7 — Consumer migration wave B (atelier)

**Steps:**

- [ ] `crates/vize_atelier_core/src/codegen/patch_flag.rs` — retained AST, local arena dies
- [ ] Remaining reparse sites from the P0-3 counter baseline (~20 in `vize_atelier_core`), checklist generated from that baseline and checked off per site
- [ ] Same differential-lane pattern as P1-6

**Acceptance:** `davinci.expr.parses` reaches its floor (== distinct expressions) on the corpus — asserted in CI from the profiler export; parity holds.
**Deps:** P1-5 (parallel with P1-6).

## P1-8 — Delete the fast/slow scanner split

**Steps:**

- [ ] Pre-deletion differential run: byte-scanner (`identifiers/fast.rs`) vs retained-AST walk over the whole corpus — committed report proving agreement (or documenting scanner bugs the AST walk fixes; those are waiver-reviewed)
- [ ] Delete `identifiers/{fast,slow}.rs` and the dispatch; single AST-walk implementation remains
- [ ] Bench check: if the scanner was faster on `stress-*` fixtures, the walk must close the gap before deletion merges (measured, not assumed)

**Acceptance:** grep zero for the deleted modules; corpus parity; croquis benches hold or improve.
**Deps:** P1-6, P1-7.

## P1-9 — Identifier prefixing as AST transform

**Steps:**

- [ ] Replace string rewriting in `crates/vize_atelier_core/src/transforms/transform_expression/{prefix,rewrite,nesting}.rs` with an AST-level transform over retained expressions (scope-aware via croquis bindings) and span-preserving emission
- [ ] Source-map assertions for rewritten identifiers added to the P0 fixture set
- [ ] **Waiver budget: zero.** Prefixing output is the most visible codegen surface; any corpus diff is a bug in this task

**Acceptance:** corpus byte parity on dom/vapor/ssr; new source-map assertions green.
**Deps:** P1-7.

## P1-10 — Node strings to `&'a str` / atoms; delete manual `Drop`s

**Steps:**

- [ ] Interner in `vize_carton` (arena-backed atoms for names appearing repeatedly: tags, directive names, well-known attrs); node fields (`name`, `tag`, `content`) become `&'a str` slices or atoms — per-field decision recorded in the PR
- [ ] Collapse the P1-1 two-pool handle: flip `vize_carton::{Box, Vec}` aliases to the oxc arena types (their no-`Drop` const assert passes once nodes are string-free), delete the `bump` pool from `Allocator`, and retire the `Bump`/`BumpString`/`BumpVec` re-exports
- [ ] Delete `ensure_sufficient_stack` `Drop` impls: `crates/vize_relief/src/relief/elements.rs`, `relief/control_flow.rs`, `crates/vize_atelier_vapor/src/ir_drop.rs` (arena drop is free once nothing owns heap strings)
- [ ] `stress-deep.vue` fixture passes without the stack guard (the guard's reason is gone, prove it)
- [ ] The `docs/content/architecture/performance.md` interning claim becomes true — note for P1-12

**Acceptance:** grep zero for manual `Drop` on node types; alloc counts drop (pin the number as the new ratchet); corpus parity.
**Deps:** P1-3.

## P1-11 — Arena reuse across files

**Steps:**

- [ ] Pool in the CLI batch path (`crates/vize/src/commands/build/`): per-rayon-worker `Allocator` reset between files (`Allocator::reset()`, available in the pinned oxc_allocator), not reallocated
- [ ] **Lifetime contract documented and enforced:** every arena-backed value is consumed or converted to its owned form (stage artifacts, cached results, diagnostics) _before_ reset — caches never hold `&'a` references and never pin an arena (see the architecture arena/cache contract)
- [ ] Escape check: asan/miri lane over the pool (nothing borrows across `reset`), plus a `#[cfg(debug_assertions)]` arena-generation counter that panics on cross-file survivals; includes a **resident-cache reset scenario** (cache populated → arena reset → cache read) proving cached data is owned

**Acceptance:** peak RSS on the corpus batch drops (pin as ratchet); asan/miri lane green.
**Deps:** P1-10.

## P1-12 — Performance-doc truth pass

**Steps:**

- [ ] `docs/content/architecture/performance.md`: every claim (interning, arenas, allocation behavior, string handling) rewritten to match shipped code, with numbers from this phase's benches; the stale pre-Davinci claims deleted

**Acceptance:** review point — maintainer signs off claims against `bench/results/davinci/`.
**Deps:** P1-10, P1-11.

## P1-13 — Phase exit

- [ ] Corpus compile parity: byte-identical, waiver ledger empty
- [ ] `davinci.expr.parses` satisfies the P1-5 counter law (zero re-parses; == or ≤ distinct expressions per the chosen policy) asserted in CI
- [ ] Compile bench improvement ≥ target pinned at phase start (set from P0-3's double-transform + reparse baselines; record the target in `budgets.toml` before P1-5 merges)
- [ ] Alloc count / peak RSS improvements pinned as new ratchet baselines
- [ ] Scanner split, string-rewrite prefixing, manual `Drop`s: deleted (grep zero)
- [ ] In-phase fallback flags removed (charter #26)
