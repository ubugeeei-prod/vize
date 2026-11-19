# Davinci completion plan for T2–T6 (excluding gates owned by other agents)

Base: HEAD `4a0fb417ed` (2026-11-19). I made no edits. With `--issues` the gate tool reports 17/45, matching your count.

## 0. Findings that cut across gates

1. **Size of the legacy code.** The 10 legacy crates hold about 293k LOC: relief 4.3k, armature 12.2k, croquis 70.2k, croquis_cf 31.1k, atelier_core 33.3k, dom 5.7k, sfc 72.3k, ssr 17.6k, vapor 32.3k, jsx 13.8k. The level crates hold about 221k.

2. **Legacy references per product** (lines/files under `src/`):

   | Product | croquis | relief | armature | atelier_sfc | atelier_jsx |
   |---|---|---|---|---|---|
   | canon | 520/285 | 109/68 | 231/92 | 79/44 | — |
   | patina | 132/89 | 324/267 | 59/48 | 108/65 | 137/80 |
   | maestro | 117/54 | 39/29 | 37/33 | 94/48 | 19/10 |

   Glyph has none.

3. **The issue bodies understate the work.**
   - #6849 says `virtual_ts` is 29.9k lines; today `crates/vize_canon/src/virtual_ts` is 80.9k and `crates/vize_canon/src/batch` is 125.8k.
   - #6850's counts have grown: croquis 105→117, atelier_sfc 79→94, armature 30→37.
   - #6848 says "230 relief/armature files"; today relief alone is 267 files.

4. **Crates outside every `products.*` gate still block `legacy.deleted`.** These are the `vize` CLI, `vize_vitrine` (NAPI/WASM), `vize_curator`, `vize_doctor`, `vize_musea` and `vize_incremental`. The level crates also keep legacy dev-dependencies as differential oracles: `vize_l0`, `vize_l1`, `vize_l1_to_l2` (including `vize_atelier_dom` with `legacy-differential`), `vize_l2` and `vize_l2_to_l3`.

5. **The perf ratchet collides with deletion.** 84 of the 101 rows in `/Users/nishimura/Source/github.com/ubugeeei-prod/vize/docs/davinci/plan/instruction-budgets.toml` are named after legacy stages (armature_parse/tokenize, croquis_analyze, atelier_*). That file says "rows may never disappear". Needs a maintainer rule (section 5).

6. **The perf agent is speeding up legacy paths.** Recent examples: #13348/#13351/#13356/#13358/#13359 (Patina legacy dispatch), #13350/#13354 (Canon), #13349/#13352/#13355 (Maestro). This raises the bar each native switch must meet, and it touches the same files as the linter, typechecker and LSP lanes.

7. **The new layout rule overlaps my L3/L4/JSX/TSRX slices.** `/Users/nishimura/Source/github.com/ubugeeei-prod/vize/docs/davinci/decisions/2026-11-17-input-neutral-levels.md` belongs to the `levels.inputs` owner. It introduces `vize_l2::module::ModuleDef` and one L4 module assembler, and it removes `vize_l4::{lang::tsrx, jsx}` and `vize_l3::jsx`. The 147 owed entries sit in `vize_l4/src/targets` (33+5), `vize_l3/src/decision` (17) and `vize_l4/src/lang` (7).

### Where docs claim more than the code shows
- **`semantic-engine.md`** promises "one typed query API, demand-driven". `vize_l2` has no `Demand` type. The only query surface is `/Users/nishimura/Source/github.com/ubugeeei-prod/vize/davinci/vize_l2/src/file/query.rs` plus `query/template*.rs`, 674 LOC. Products call it at 5 sites (maestro 4, canon 1).
- **`architecture.md`** says JSX lowers into the same ops as templates. That is true for 169 of 226 oracle roots only. All JSX emission still goes JSX → Relief → atelier_core (`crates/vize_atelier_jsx/src/lib.rs:64`).
- **`levels.container` (closed).** The parser really is on L1 (`crates/vize_atelier_sfc/src/parse.rs`). But the `SfcDescriptor` type still lives in `/Users/nishimura/Source/github.com/ubugeeei-prod/vize/crates/vize_croquis/src/sfc/types.rs:16`. `vize_incremental` imports `vize_croquis::sfc::parse_sfc` (`crates/vize_incremental/src/descriptor.rs:14`, `artifact.rs:14`). Croquis cannot be deleted until this type moves.
- **#6872's "stage queries unused by the LSP" is still true.** `vize_incremental::{l1_block,l2_page,file_artifacts}` have 0 uses in maestro; it uses only `parse_descriptor`.
- **The roadmap says plan files are normative**, but some phase-4 tasks build on `vize_croquis::facts` (`phase-4-tasks.md:87`, `:176-177`). That contradicts the #6844 "no Croquis adapter" decision.
- **#6839's "duplicated decision logic" is still true.** `vize_l3::placement` has 0 external consumers. The production DOM emitter `davinci/vize_l4/src/targets/dom/artifact` (25k LOC) still decides hoisting and caching from L2 `static_facts`; only `patch.rs` imports `vize_l3::lattice`.

### Parity and perf protocol for every slice
- **P-bytes:** compiler snapshots, `crates/vize_atelier_sfc/tests/davinci_production_reach.rs` (floors in `docs/davinci/plan/reach-budgets.toml`: DOM 347/351, SSR 349/351, Vapor 203/351), the Vue differential corpus, `tests/_fixtures/differential/<product>`, and the weekly Real Project Matrix.
- **P-diag** (linter, typechecker, LSP): mapped diagnostics identical, or every difference classified with evidence.
- **P-instr:** `level-instruction-counts.yml` (Callgrind, three identical runs, ceilings only fall). Also the SFC and formatter registries, and the `davinci_fact_query_*` rows.
- **P-mem/CPU/wall:**
  - all products: `level-memory-bench.yml`, `pr-performance-observation.yml`;
  - compiler: `compile-bench.yml`;
  - typechecker: `check-bench.yml`, `cold-typechecker-scale.yml`;
  - LSP: `lsp-dual-server-benchmark.yml`, `lsp-vs-volar`, and the R1 ceilings (keystroke p95 46 ms, RSS 337/316 MiB, binary 59 MiB).
- **Rule:** a native fact or route must replace the legacy computation in the same PR, never run beside it. Otherwise CPU and RSS go up while both paths coexist. Where a legacy analysis still serves other consumers, build it lazily, only when a still-legacy consumer is enabled.

## 1. T2

### `levels.query` (#6871)
**Current state:** see the flags above. Croquis is still the de facto semantic API: `Analyzer` 322 sites, `Croquis` 581, `ScopeKind` 246, `ComponentUsage` 154, `Croquis.macros` 217 (`node tools/support/compat/davinci/croquis-consumers.ts --summary`). The gate text ("products read semantics through one L2 query API") cannot close before the T4 product migrations. The T2 deliverable is an API that is complete.

**Slices:**
- **Q1 – design record.** Entry type `vize_l2::query::Model<'f,'a>` over `FileArtifact`, `ArtifactParts` and the `framework::vue` facts. Method families: position, resolution, scope chain, binding kind, Vue contract, template usages. Borrowed and allocation-free, typed refusals, follows #6872's span-sorted snapshot index. Needs sign-off from the LSP and Canon owners.
- **Q2 – `vize_l2::query` façade** over the existing `file::query`; move the 5 product call sites to it. Zero behaviour risk; P-instr on `davinci_fact_query_*`.
- **Q3 – Vue queries.** Binding kind, macro calls, props/emits/models/slots/expose, options members, all from the #6844 S2/S3/S6/S7 facts. Additive, `vize_l2` only.
- **Q4 – template-usage queries (Croquis family 11), about 3 PRs.** Component usages with resolved binding and passed or spread props; listeners and slot usages; template expressions with kind and scope. These must be derived from L2 ops and template sites with no new walk. Oracle: a dev-test exact-equality comparison against Croquis on the repo corpus, with Croquis defects classified as in #6844 S7.
- **Q5 – snapshot position index** for the LSP: span-sorted, built once per snapshot.
- **Q6 – mechanical check.** A tooling test forbidding `vize_l2::{op,artifact,file::records,walk}` imports from patina, maestro and canon, with a shrinking allowlist. Needs maintainer approval because it changes the gate definition.

**Depends on:** S2–S7 for Q3. The gate closes as LN3, TC5 and LSP3 drain.

### `levels.script` (#6844)
**Current state.** Following the slice table in `/Users/nishimura/Source/github.com/ubugeeei-prod/vize/docs/davinci/decisions/2026-11-09-6844-native-script-inventory.md`:
- **Done:** S1 #12396, S2 #12411, S3 #12419/#12429, S4 #12442 + #12566 (setup route native), S7 #12699/#13192/#13303.
- **S6 partial (#12575):** `ScriptCompileContext` still bridges to Croquis (`crates/vize_atelier_sfc/src/script/context.rs:17-18,181`, `define_model_metadata.rs:24`).
- **S5 not started:** `is_component_registered` is still Croquis (`crates/vize_croquis/src/croquis/model.rs:109`), read from `crates/vize_atelier_core/src/lane/element.rs:120`.
- **S8–S10 not started.**
- 24 compiler files in `vize_atelier_sfc` and 5 in the `atelier_core` lane still import croquis.

**Slices:**
- **S5:** component-registration facts in `vize_l2::framework::vue`, plus switching the atelier_core/dom lanes. P-bytes; P-instr on the atelier_dom/core rows.
- **S6a:** in-file macro type arguments (interfaces and type literals giving runtime props/emits types). Oracle: Croquis `PropDefinition`/`EmitDefinition`.
- **S6c:** switch `ScriptCompileContext` macro processing to the native contract and delete the `add_model_to_croquis` bridge. P-bytes plus the Vue differential.
- **S6b:** cross-file type resolution through the project model, lazily, replacing `script/context/type_world` and `ResolvedTypeWorld`. High perf risk because the CLI uses `begin_type_resolution_batch`: needs compile-bench and check-bench. Depends on PM1.
- **S4b:** remaining reactive reads in the atelier_core lane (`lane/context.rs`, `steps/expression/{prefix,inline_handler}.rs`).
- **S9:** setup context, provide/inject and the effect graph, plus project-level facts. This replaces `vize_croquis_cf` (31k LOC) for the CLI, curator, doctor, vitrine and maestro. Depends on Q4 and PM1–PM3. It is the largest remaining script item.
- **S8** is done inside LN3/LSP3; **S10** inside TC5.

### `levels.jsx` (#6885)
**Current state:** delivery steps 1–4 are merged (#12570, #13133, #13160, #13169, #13183, #13190), and 169/226 oracle roots run on `/Users/nishimura/Source/github.com/ubugeeei-prod/vize/davinci/vize_l1_to_l2/src/framework/vue/jsx` (2,884 LOC).
- The only caller is `vize_atelier_jsx` (`finder.rs:22`, `l2.rs:63-67`).
- Remaining refusals:
  - spread children (`jsx/children.rs:212`);
  - value `&&`;
  - `_` modifiers (`VIZE.event_underscore_modifiers=false`);
  - mixed text and slot return values;
  - `v-slot`;
  - `render-expr`.
- `BABEL` and `VUE_JSX_VAPOR` exist only as feature constants.

**Slices** (each uses the record's three checks: L2 dump equals the Relief projection on admitted rows; byte-identical output apart from the listed fixes; zero JSX fallback on admitted rows):
- **J2b** `ui.and` and non-element branches.
- **J3b** `_` modifiers (#12313).
- **J4b** mixed text and slot return values as `ui.jsx-child` (#12322).
- **J5** `ui.child-spread` and `ui.render-expr`. These are new `vize_l2::op` variants, so L2→L3 and L4 must also change; run this one serially with Lane B.
- **J6** `BABEL` (#12324, #12334).
- **J7** RunOnce/`VUE_JSX_VAPOR` on native Vapor (#12311/#12312/#12323). Needs L4-5.
- **J8** SSR. Needs L4-4.
- **J9** products read `ComponentDef`, then delete `atelier_jsx::lower`/`::l2`/Relief. Needs LN, TC3 and maestro JSX work.

Add a JSX compile instruction row before J7: there is none today.

## 2. T3

### `levels.l3` (#6839)
**Current state:** see the flags. On top of that:
- Native SSR (`crates/vize_atelier_ssr/src/l4`, 8.9k) uses the L3 partition facts.
- Native Vapor (`crates/vize_atelier_vapor/src/l3`, 9.8k) converts back to legacy `RootIRNode` (`compile/native.rs:5,17`).
- The vocabulary-neutrality review has not been done.

**Slices:**
- **L3-1 – record:** which decisions move into L2→L3, keyed by L2 NodeId: static partition, hoist placement, cache slots, patch flags, SSR string plan, effect order. Also neutral names vs a `vapor` dialect.
- **L3-2:** `vize_l2_to_l3` computes DOM placement, with a dev-test equivalence check against `static_facts`. Additive and off the hot path.
- **L3-3:** the DOM artifact emitter reads L3 placement, one family per PR (hoists, then cache slots/once/memo, then patch flags). Each PR deletes the matching `static_facts` pass so the instruction rows hold. P-bytes plus the reach floors.
- **L3-4:** SSR string plan moves into `vize_l3::decision::ssr`.
- **L3-5:** Vapor effect grouping and ordering, handled together with L4-5.
- **L3-6:** OpKind rename. Dump changes only; coordinate with the `levels.inputs` owner, who is editing `vize_l3/src/decision`.

### `levels.l4` (#6840)
**Current state:**
- The DOM emitter has moved into L4 (2026-11-09 record) but still reparses retained expressions.
- The L4 SSR target is 283 LOC and the Vapor target 541. Production SSR and Vapor live in the legacy crates. Native SSR has 63 legacy references: atelier_core `EmitDocument`/`RuntimeHelper` and armature `FrozenElements`, and every SSR compile parses with legacy armature first (`compile/inner.rs`). Native Vapor emits through the legacy generator (6.8k + 1.1k).
- There are two `EmitDocument` types: `crates/vize_atelier_core/src/codegen/document.rs:50` and `davinci/vize_l4/src/write/document.rs:56`.
- Style emission is still in `atelier_sfc::{style,css}`.

**Slices:**
- **L4-1:** one `EmitDocument` (delete the atelier_core copy). P-bytes on all three backends; TS-31 source maps must stay 35/35, 13/13 and 11/11.
- **L4-2:** native SSR uses `vize_l4::runtime` helpers.
- **L4-3:** native SSR reads the L1 surface; the legacy parse runs only on refusal. Expected perf gain.
- **L4-4:** move `atelier_ssr/src/l4` into `vize_l4::targets::ssr`. Pure move, same precedent as the DOM move.
- **L4-5:** Vapor JS written directly from L3, 3–4 PRs by construct family. The L3→`RootIRNode` adapter stays as a dev oracle until parity, then is deleted. Rows `atelier_vapor_generate`/`compile` must fall.
- **L4-6:** move the native Vapor builder out of the atelier crate.
- **L4-7:** raise Vapor reach from 203/351, one refusal bucket per PR (`phase-3-records/p3-17.md`).
- **L4-8:** remove the DOM reparse.
- **L4-9:** converge the artifact DOM emitter and the L3-driven DOM target into one (after L3-3).
- **L4-10:** L4 style writer (`11550` step 5).
- **L4-11:** module assembly through the single assembler, after the `levels.inputs` `ModuleDef` work lands.

**Order:** L4-1 → L4-2 → L4-3 → L4-4 can start now. L4-5 needs L3-5. Keep out of `vize_l4::{module,lang,jsx}` until the `levels.inputs` owner lands its changes.

## 3. T4

### `products.formatter` (#6847) – cheapest to close
**Current state:**
- Glyph's normal dependencies are `vize_l0` and `vize_l1` only; its product allowlist is empty and its source has 0 legacy references.
- Its directive grammar comes from `vize_l1` (`DirectiveName`, `DirectiveKind`); history #6882 is closed.
- Residue that is not on L1:
  - an expression reparse in `/Users/nishimura/Source/github.com/ubugeeei-prod/vize/crates/vize_glyph/src/template/literal_spans.rs:109,112`;
  - its own oxc parse in `script/format.rs:38`;
  - memchr byte scanning in `template/formatter.rs:300`, `whitespace_significant.rs` and `preserved_text.rs`;
  - an opt-in `native_doc/` route (5.8k LOC) beside the default `template/` route (6k);
  - plan task `phase-4-tasks-last.md:120` is still open.
- Last activity on the issue was 2026-10-04.

**Slices:**
- **F1:** audit record.
- **F2:** an L1 format profile for Programs (keeping `preserve_parens` / `parse_for_format`), consumed by `script/format.rs`.
- **F3:** `literal_spans` uses retained L1 spans (perf gain).
- **F4:** replace the byte scanners with L1 token and trivia spans, about 3 PRs.
- **F5:** promote or delete `native_doc` (maintainer).
- **F6:** close with evidence.

Proof: the formatter fixture gate, its registered differential verifier and the formatter instruction budgets.

### `products.linter` (#6848)
**Current state** (trait implementations in `crates/vize_patina/src/{rules,native}`):

| Trait | Count |
|---|---|
| `Rule` (Relief) | 271 |
| `ScriptRule` | 98 |
| `MarkupRule` (facade) | 87 |
| `CssRule` / `StyleRule` | 9 / 2 |
| `MuseaRule` | 5 |
| `Native*` (opt-in) | 16 |

- The default SFC markup engine already uses `MarkupDocument::from_l2` (`linter/engine/sfc/facade.rs:161`). Relief is still built at `linter/engine/markup.rs:56` and by 3 rules.
- The allowlist has 11 edges.

**Slices:**
- **LN1:** generated substrate ledger (each rule → L1 syntax / L2 semantic / L1 script / L1 css), with a staleness test.
- **LN2:** turn `native/` into the default L1 syntax engine, then migrate rules in waves of about 20, by directory. One agent owns `linter/engine/*`. The Relief parse disappears only after the last wave, so no wave may add a second L1 parse; reuse the tree the L2 facade already builds.
- **LN3:** semantic rules on Q3/Q4 (the 55 Croquis-reading files).
- **LN4:** `ScriptRule`s on the L1 Program and L2 file facts, with JSX/TSX through the same registry.
- **LN5:** remaining css rules.
- **LN6:** fixes become `SpanEdit` (see FX2).
- **LN7:** delete the Relief engine and the `atelier_jsx` dependency (after J9).

Proof: P-diag on Patina fixtures and the lint differential, Real Project Matrix lint baselines, Patina instruction rows. The default switch needs #6881 plus the linter harness, both owned by other agents.

### `products.typechecker` (#6849) – longest critical path
**Current state:** L4 `targets/ts` is 1,013 LOC (#13148 JSX/TSX modules, #13153 if/for, #7547/#7624 originals, used by `corsa_bridge/native_vue*.rs`). Rollout step 2 of `/Users/nishimura/Source/github.com/ubugeeei-prod/vize/docs/davinci/decisions/2026-11-10-6849-l4-ts-projection.md` is partial; steps 3–6 have not started; the harness native adapter is `null`.

**Slices:**
- **TC1:** target core (link roles, directive ranges, access provider, helper table plus a tooling test).
- **TC2:** JSX families, following J-steps.
- **TC3:** opt-in seam in `crates/vize_canon/src/batch/virtual_project/jsx_build.rs`, plus a differential lane in `typecheck-corpus-pair.yml`. Wiring goes through the Canon perf owner, as the decision requires.
- **TC4:** TSRX projection.
- **TC5:** SFC. Macro respelling (S2/S6) and context (S3) first, then component/slot/model, then Options API, then Vue 2.
- **TC6:** default switch per family, gated by #6879, no unclassified differential rows, and cold CPU/RSS ≤ today.
- **TC7:** delete the old projection per family, and merge `crates/vize_maestro/src/virtual_code`.

Extra metric: projected bytes per SFC must not grow, because tsgo time scales with projection size (2026-11-03 #11649 record).

### `products.lsp` (#6850, #6872)
**Current state:** maestro is 172k LOC; there are 27 armature `Parser` sites outside tests across about 35 files, and 3 `Analyzer::new` calls. `vize_incremental` itself imports croquis and relief. The allowlist has 20 edges.

**Slices:**
- **LSP1:** `vize_incremental` on the L1 container; incremental-equivalence CI stays green.
- **LSP2:** retain per-open-document L1/L2 artifacts in owned form, plus `SfcSummary` for every file. RSS ceilings apply.
- **LSP3:** route requests through the Q5 snapshot, migrating one feature family per PR: hover, completion, definition/references/rename (`template_scope/*`), symbols, folding, code actions, diagnostic collectors. Each PR removes its parser sites.
- **LSP4:** `virtual_code` moves into TC.
- **LSP5:** remove croquis/croquis_cf (after S9).

Proof: LSP response fixtures on all three TS backends, #6883 fixtures, keystroke p95.

### `products.project` (#6874)
**Current state:** `/Users/nishimura/Source/github.com/ubugeeei-prod/vize/crates/vize_carton/src/config/project.rs:12` exists, and the LSP owns one applied model (#10124). Still separate: CLI/editor loading (about 14 `load_config_*` variants in `vize_carton/src/config/loader*.rs`), workspace folders, invalidation.

**Slices:**
- **PM1:** one owned config snapshot; the variants become accessors.
- **PM2:** LSP workspace folders and CLI root/tsconfig selection share one rule.
- **PM3:** one invalidation API.
- **PM4:** unify tsconfig-graph ownership.

Proof: config loader tests and the CLI/LSP matrix.

### `products.fixes` (#6876)
**Current state:** the provider `/Users/nishimura/Source/github.com/ubugeeei-prod/vize/davinci/vize_l1/src/edit.rs` has only two consumers, both in maestro (`source_project/edit.rs`, `code_action/versioned_lint_edits.rs`). Patina's `Fix`/`TextEdit` (`crates/vize_patina/src/diagnostic/types.rs:58,99`) is a separate type.

**Slices:**
- **FX1:** one fix type made of `SpanEdit`s plus the document version (record).
- **FX2:** Patina uses it; the CLI/NAPI JSON shape does not change.
- **FX3:** maestro code actions use it.
- **FX4:** Canon/compiler quick fixes.
- **FX5:** any Glyph rewrite that changes semantics becomes a lint autofix.

### `inputs.sass` / `scss` / `less` / `stylus` (#11550–#11553, one pipeline)
**Current state:**
- **Done:** step 1 of the style-language audit record (`vize_l1/src/css/{lang,structure,classes,bind}`, #12426/#12437/#13231; `vize_canon::css_classes` is gone) and part of step 4 (Patina css rules on every language, #13222/#13234).
- **Not done:**
  - step 2 (typed `$style`) – verify;
  - steps 3–4 for Maestro and Glyph;
  - step 5: the L4 writer, CLI preprocessing still using legacy `parse_sfc`, `vite_plugin` scoping.
- The dialect ids are already in `deletion-readiness-policy.json`.

**Slices:**
- **ST1:** `$style` typing (with TC).
- **ST2:** Maestro style lexers onto L1 structure.
- **ST3:** Glyph `style/preprocessor_layout.rs` and `quotes.rs` onto L1 spans.
- **ST4:** equals L4-10, then CLI preprocessing on L1 views, then NAPI and bundler post-transform.
- **ST5:** remaining css rules.
- **ST6:** release.

### `inputs.jsx` / `inputs.tsx` (#11554/#11555)
These close as J1–J9, TC2/TC3/TC6, LN4 (JSX), the maestro JSX feature families and Glyph JSX finish, plus zero-fallback rows and a release.

### `inputs.vapor` (#11556)
Closes with L3-5, L4-5..7, J7, and run-once roots in the TS projection, which are refused today until L2 carries the execution model.

## 4. T5 / T6

### `legacy.deleted`
Depends on `legacy.zero-fallback` holding for 30 days, the guard `tools/support/compat/davinci/legacy-deletion-guard.ts`, and exact-SHA readiness audits.

**Order:**
- **D0:** home for the compiler shell (maintainer).
- **D1:** croquis_cf (after S9).
- **D2:** atelier_jsx (after J9, LN7, TC7).
- **D3:** atelier_sfc. Its shell moves per D0, and `SfcDescriptor` moves out of croquis.
- **D4:** ssr, vapor, dom (once reach is 100% and zero-fallback holds).
- **D5:** atelier_core.
- **D6:** croquis.
- **D7:** armature and relief.
- **D8:** remap the instruction rows, and turn the level crates' legacy dev-dep oracles into frozen golden fixtures.

### `inputs.tsrx` (#6858)
**Current state:** 5,765 LOC.
- **Native today:** L1 `markup::tsrx`, `vize build` (#13138) and `vize lint` markup regions (#13127).
- **Misplaced:** L2 lowering sits in `vize_l2::lang::tsrx`, and the compile entry sits in `vize_l4::lang::tsrx` (DOM only).
- **Open:** #13128 and #13140; every item in the #6252 checklist is unticked.

**Slices:**
- **X1:** NAPI and bundlers (#13140).
- **X2:** lint host script rules (#13128).
- **X3:** SSR, after the `ModuleDef` work.
- **X4:** Vapor, after L4-5.
- **X5:** TC4 plus the LSP.
- **X6:** `@try` and scoped styles – implement or diagnose (maintainer).
- **X7:** perf, docs, release.

## 5. Parallel lanes and dispatch

**Lanes and file ownership:**
- **A – L2 semantics:** `vize_l2/src/{query,framework}`, `atelier_sfc/src/script*`, `atelier_core/src/lane`.
- **B – backends:** `vize_l3`, `vize_l2_to_l3`, `vize_l4/src/{targets/{ssr,vapor,dom/artifact},write,runtime}`, `atelier_{ssr,vapor,dom}`, `atelier_core/src/codegen`.
- **C – JSX and TSRX:** `vize_l1_to_l2/src/framework/vue/jsx`, `atelier_jsx`, `vitrine` tsrx, Patina tsrx.
- **D – typechecker:** `vize_l4/src/targets/ts`, `vize_canon`.
- **E – linter and fixes:** `vize_patina`, `vize_l0` diagnostics.
- **F – formatter, project, LSP:** `vize_glyph`, `vize_carton/src/config`, `vize/src/commands`, `vize_maestro`, `vize_incremental`.

**Conflicts to manage:**
- B with the `levels.inputs` owner, on `vize_l3/src/decision` and `vize_l4/src/targets`.
- J5 with B, because of the new `vize_l2::op` variants; run it serially.
- D, E and F with the perf agent, on canon, patina and maestro.
- E and F with the #6851 owner, on `docs/davinci/plan/product-dependency-allowlists/*.json`.

**Recommended dispatch (6 agents; first items can start today):**
1. **A:** Q1 → Q2 → S5 → S6a → S6c → Q3 → Q4 (×3) → S6b (after PM1) → S9.
2. **B:** L3-1 → L4-1 → L4-2 → L4-3 → L4-4 → L3-2 → L3-3 → L4-5 → L4-7 → L4-10.
3. **C:** J2b → J3b → J4b → J6 → J5 (sequenced with B) → X1 → X2; J7/J8/X3/X4 once B delivers.
4. **D:** TC1 → TC2 (following C) → TC3 → TC4 → TC5 (after Q3); coordinate with the Canon perf owner.
5. **E:** LN1 → FX1 → FX2 → LN2 waves → LN5 → LN4 → LN3 (after Q3/Q4).
6. **F:** F1–F6 (closes the gate early) → PM1–PM3 → LSP1 → LSP2 → LSP3 families → ST2/ST3.

With only 4 agents, merge C into B and F into E. That pushes JSX past the first L4 slices.

## 6. Decisions only you can make
1. **Compiler shell after deletion.** Where the public `compile_sfc`, the NAPI types, the `vite_plugin` helpers and CSS compile go. Also confirm that a rename-only change (for example `atelier_sfc` → `vize_compiler`) does not satisfy `legacy.deleted`, even though the measured gate only checks crate names.
2. **Instruction ratchet vs deletion.** A rule that maps each of the 84 legacy-named rows to a native row that keeps the same ceiling.
3. **"No regression" while legacy and native paths coexist.**
   - Is binary size and RSS judged strictly per PR, or against the ceilings (59 MiB server, 337/316 MiB) with a net reduction at deletion?
   - Purely additive native paths grow the binary every time.
4. **`levels.query` acceptance.** API completeness plus the Q6 import rule? And are charter #8's demand declarations and undeclared-access detector in scope?
5. **Glyph.** Promote or delete `native_doc`. Does an oxc_formatter parse through an L1 profile count as "formats from L1"?
6. **One DOM emitter.** Confirm the L2 `static_facts` decision passes are deleted (not kept) once L3 placement is consumed, and the artifact and native DOM targets merge.
7. **TSRX scope.** Implement `@try`, sibling-scoped styles and server modules, or accept explicit unsupported diagnostics for the gate.
8. **Meaning of "Vapor SSR"** in #11556.
9. **Owner and gate for the uncovered crates:** CLI, vitrine, curator, doctor, musea, incremental.
10. **Stale plan tasks.** Mark the phase-4 tasks that build on `vize_croquis::facts` as superseded by the #6844 "no adapter" decision.
11. **Allowlist ownership.** Migration PRs shrink the product allowlists in the same PR, as #6844 decision 5 already says, and the #6851 owner keeps the tooling. Also decide how the perf agent's legacy-path work is sequenced against E, F and D.

### Critical Files for Implementation
- /Users/nishimura/Source/github.com/ubugeeei-prod/vize/davinci/vize_l2/src/file/query.rs
- /Users/nishimura/Source/github.com/ubugeeei-prod/vize/davinci/vize_l1_to_l2/src/framework/vue/jsx.rs
- /Users/nishimura/Source/github.com/ubugeeei-prod/vize/crates/vize_atelier_ssr/src/compile/inner.rs
- /Users/nishimura/Source/github.com/ubugeeei-prod/vize/crates/vize_atelier_vapor/src/compile/native.rs
- /Users/nishimura/Source/github.com/ubugeeei-prod/vize/davinci/vize_l4/src/targets/ts.rs
## Maintainer decisions (2026-11-19, final)
- §6.1 Compiler shell: **keep `vize_atelier_sfc`** as the public shell (compile_sfc, NAPI types, vite_plugin helpers, CSS compile), emptied to wiring only. The `legacy.deleted` gate must be adjusted to allow `vize_atelier_sfc` as a shell that holds no compile logic (record a decision).
- §6.2 Instruction rows: **keep the legacy row names**; repoint them at the native code paths that do the same work. Ceilings only fall.
- §6.3 No-regression while coexisting: strict per PR on every metric (native replaces legacy in the same PR).
- §6.4 `levels.query`: **complete `vize_l2::query` API + Q6 import rule** (tooling test forbidding patina/maestro/canon from importing L2 internals, shrinking allowlist). Demand declarations not required.
- §6.7 TSRX: **implement all** — `@try`, sibling-scoped styles, server modules, with parity against the reference compiler.
