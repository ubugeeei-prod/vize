# products.dependencies (#6851): legacy edge inventory

Measured 2026-11-19 at `1cdd5410f8` (origin/main). The gate counted 37 allowlist
rows: Canon 6, Maestro 20, Patina 11, Glyph 0. Several rows repeat across
roots, so there are **19 distinct legacy edges** (`from -> to`):

| from               | to (legacy)                                                                                              | rows in allowlists                      |
| ------------------ | -------------------------------------------------------------------------------------------------------- | --------------------------------------- |
| `vize_canon`       | armature, atelier_core, atelier_jsx, atelier_sfc, croquis, relief (6)                                    | Canon, Patina (via canon), Maestro (via canon) = 18 |
| `vize_patina`      | armature, atelier_jsx, atelier_sfc, croquis, relief (5)                                                  | Patina, Maestro (via patina) = 10       |
| `vize_maestro`     | armature, atelier_jsx, atelier_sfc, croquis, croquis_cf, relief (6)                                      | Maestro = 6                             |
| `vize_incremental` | croquis, relief (2)                                                                                      | Maestro = 2                             |
| `vize_musea`       | croquis (1)                                                                                              | Maestro = 1                             |

Patina inherits the Canon edges because it depends on `vize_canon` (non-wasm
target) for its type-aware rules. Maestro inherits Canon, Patina, Incremental
and Musea edges. **Canon's six edges account for 18 of the 37 rows.** A
Canon edge leaves all three files at once.

## Method

For each crate, the legacy crates were moved out of `[dependencies]` and
`cargo check -p <crate> --lib` was run. Every non-test reference to an unlinked
crate then fails to compile, so the file and site counts below are exact for
the default-feature library (`cfg(test)` code, `legacy`, `legacy-differential`
and wasm-only code are not counted). The dev-dependency form of each legacy
crate is not gated (only normal edges count), so tests may keep the legacy
crates as oracles.

`node tools/support/compat/davinci/croquis-consumers.ts --summary` gives the
symbol-level Croquis consumption, per family. It agrees with the families below.

## Per-edge inventory

"Native today" names the level API a consumer would switch to. "Blocker" is the
issue whose work must land first. A route switch is a change that replaces
legacy behavior with native behavior. Each one needs a default-off flag plus the
differential, fix-history and real-project evidence before the default flips.

### Patina (direct edges: 5)

| edge                                | non-test lib sites                                      | API used                                                                                                                                                                                                                                       | native today                                                                                                                                                                                                                        | blocker                                                                    |
| ----------------------------------- | ------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| `vize_patina -> vize_relief`        | 253 files / 288 sites (rules/vue, a11y, opinionated, ssr, visitor, context, markup, html_content_model, linter/config) | The `Rule` trait hooks take Relief nodes (`run_on_template(&RootNode)`, `enter_element(&ElementNode)`, `check_directive`, `check_for(&ForNode)`, `check_if(&IfNode)`, `check_interpolation`), `SourceLocation`, `BindingType`, `CustomElementMatcher` | `MarkupRule` over the L2 facade (`crate::markup::L2Markup`, P4-7). `NativeTemplateRule`/`NativeSfcTemplateRule`. Rules that already have `as_markup_rule` also run on L2                                                          | #6848: every remaining Relief-hook rule must move to `MarkupRule`/native, then the `Rule` hook surface and `visitor` are deleted |
| `vize_patina -> vize_armature`      | 8 files / 8 sites                                       | Re-parses templates into Relief: `html_content_model/build.rs` (`Parser::with_options_and_template_syntax`, Quirks), `plugin_batch/production.rs`, `rules/opinionated/vue/no_root_v_if.rs`, `script_unused/art_use.rs`. `patterns::attribute_source_span` (4 rules), `parser::retain_slot_parameters_in` | L1 `vize_l1::markup` surface parse (authored structure, no tree repair) and the L2 facade's authored L1 projection (`authored_document_skeleton`). There is no native `attribute_source_span`; L1 attribute events carry value spans | #6848 (same rules as the Relief edge)                                      |
| `vize_patina -> vize_croquis`       | 88 files / 133 sites (rules/script 20, rules/vue 12, opinionated 11, native_type_aware 8, plugin_batch 4, …) | `Croquis` (macros, bindings, unused bindings, options descriptor, template references), `ResolvedTypeWorld`, `script_parser::{collect_options_descriptor, parse_script_setup}`, `drawer::{TemplateReferences, parse_v_for_*, LintTemplate, strip_js_comments_with_offsets}`, `facts::*`, `naming::*`, `builtins::is_builtin_component`, `MacroKind` | `vize_l2::framework::vue`: macro table (S1), `SetupFacts::macro_calls` (S2), `SetupBinding` kinds (S3), `ComponentContract` (S6, partial), `OrdinaryFacts`/`OptionsFacts` (S7). L2 references and `file::query` for unused bindings. No consumer reads them yet | **#6844 `levels.script`**: S8 (families 6, 8 and 10 for Patina) plus S3/S6 completion; template-facing facts (family 11) are #6836/#6871; type world needs #6874 |
| `vize_patina -> vize_atelier_sfc`   | 54 files / 80 sites (linter/engine, native_type_aware, rules/musea, facts, plugin_batch, output) | `SfcDescriptor`/`parse_sfc`/`SfcParseOptions`/`Sfc*Block` (the container), and the `vize_atelier_sfc::croquis` bridge (`analyze_sfc_descriptor*`, `SfcScriptParses`, `resolve_descriptor_type_world*`, `SfcCroquisOptions`), `script::TypeSourceSnapshot` | L1 `Vue::observe_descriptor` -> `AdmittedDescriptor` (`ScriptView`, `StyleView`, `TemplateView`). It refuses custom blocks (`UnsupportedBlock`), `src`, unsupported spellings and options, so it is not a drop-in descriptor | Container: an L1 descriptor that admits custom blocks, or a decision on where the compatibility descriptor lives (see decisions). Bridge: #6844 |
| `vize_patina -> vize_atelier_jsx`   | 5 files / 8 sites (`linter/engine.rs`, `engine/jsx.rs`, `engine/batch.rs`, `markup/l2.rs`) | `parse_module`, `lower_source` (Relief root plus `l2::JsxL2Root` projection), `JsxLang`, `JsxDiagnostic`, `LowerOutput` | `vize_l1_to_l2::framework::vue::jsx` (shared L1->L2 JSX lowering, in progress) and the L1 `parse_program_once` | **#6885** step 9 ("products read `ComponentDef`s; `vize_atelier_jsx::lower`, `::l2` … deleted"). Another agent is landing steps 1–5 now |

Patina also inherits the 6 Canon edges through `vize_canon` (type-aware
rules: the checker's virtual TS document and the tsconfig snapshot).

### Canon (direct edges: 6). These also clear 6 Patina rows and 6 Maestro rows.

Almost every Canon site is in the current projection, `virtual_ts/*` (scope,
generator, expressions, props) and `batch/virtual_project/*`. #6849 replaces
that projection with the `vize_l4::targets::ts` L2 projection. Per
`docs/davinci/decisions/2026-11-10-6849-l4-ts-projection.md`, the current
projection for an input family is deleted only after that family's default
switch. So **every Canon edge is blocked on the #6849 default switch for
SFC and JSX**, and on #6828 deleting `virtual_ts`.
`virtual_ts/generator.rs` is byte-custody pinned.

| edge                              | non-test lib sites                              | API used                                                                                                                                                                                       | native today                                                                                                     | blocker                                                                                          |
| --------------------------------- | ----------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `vize_canon -> vize_croquis`      | 192 files / 339 sites (virtual_ts/scope 63, generator 44, expressions 30, batch 14, props 12) | `Croquis`, `Analyzer`/`AnalyzerOptions`, `croquis::*`, `facts::*`, `drawer::*`, `script_parser::*`, `macros::*`, `TemplateExpression`, `ScopeKind`/`ScopeData`/`ScopeChain` | L2 setup/contract facts (#6844 S2/S3/S6/S7) and the L4 TS projection's context access provider | **#6844 S10** and the **#6849** SFC default switch |
| `vize_canon -> vize_relief`       | 58 files / 85 sites (virtual_ts/scope 24, generator 16, expressions 6, batch 5) | The template AST walk (`RootNode`, `TemplateChildNode`, `ElementNode`, `PropNode`, `ExpressionNode`), `options::*`, `BindingType`                                             | The L2 ops the L4 TS target projects (table in the 6849 record)                                                  | #6849 SFC default switch                                                                         |
| `vize_canon -> vize_armature`     | 3 files / 3 sites, plus `legacy.rs` under feature `legacy` | `patterns::{MatchPattern, PatternBinding, PatternKind, attribute_source_offset}` (`virtual_ts/scope/patterns*`), `legacy::{LegacyVueVersion, LegacyDialectCapabilities, DirectiveArgStyle}` | v-for/slot patterns: L2 region declarations. Legacy version model: `vize_l0::config::VueVersion` + dialect profiles | #6849. The `legacy` feature also enables `vize_armature/legacy`, so it needs an L0/L1 home for the version model (decision) |
| `vize_canon -> vize_atelier_core` | 18 files / 25 sites                             | `TemplateSyntaxMode` (a re-export of `vize_l0::TemplateSyntaxMode`; 10 files). `codegen::document::{EmitDocument, SpanLink}` (the atelier_core document, not `vize_l4::write::EmitDocument`). `parser::parse_with_options*` (template re-parse for `@vue-ignore`/`@ts-*` directive comments, art usage, Vue compiler comments), `CompilerError`/`ErrorCode` | `vize_l0::TemplateSyntaxMode` (path swap only). `vize_l4::write::{EmitDocument, SpanLink}` (the writer the L4 target uses). Directive ranges come from the L4 target (`ui.comment` row) | `TemplateSyntaxMode`: none (hygiene). The rest: #6849 |
| `vize_canon -> vize_atelier_sfc`  | 34 files / 53 sites                             | `SfcDescriptor`/`parse_sfc`, the `croquis` bridge (`analyze_sfc_descriptor_resolved`, `merge_resolved_props_into_croquis_with_sources`, `descriptor_module_sources_are_known`), `script::{SfcTypeProject, TypeSourceSnapshot, resolve_template_used_identifiers, resolve_template_read_identifiers}`, `validate_script_setup_semantics_located`, `prepare_root_patterned_template` | L1 descriptor (see Patina); type resolution: none (#6844 S6 through #6874) | #6849 + #6844 S6/S10 + the descriptor decision |
| `vize_canon -> vize_atelier_jsx`  | 3 files / 5 sites (`jsx_build.rs`, `jsx_codegen.rs`, `jsx_codegen/collect.rs`) | `lower_source_for_typecheck`, `JsxLang`, `JsxDiagnostic`, `StyleExprSpan`                                                                                                          | L4 TS target JSX/TSX module projection (6849 rollout step 2/3)                                                 | #6849 steps 2–3 and #6885                                                                        |

### Maestro (direct edges: 6; inherited: Canon 6, Patina 5, Incremental 2, Musea 1)

| edge                                 | non-test lib sites                                              | API used                                                                                                                                                                                                  | native today                                                                                                                                                         | blocker                                                                                    |
| ------------------------------------ | --------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| `vize_maestro -> vize_armature`      | 28 files / 30 sites (ide/template_scope 5, rename 4, diagnostics 4, hover 3, server/state 2, completion 2, virtual_code 2) | Per-request template re-parse into Relief (`parse`, `parse_document` for petite-vue), `tokenizer::entity_decode::try_decode_entity`, `patterns::{MatchPattern, PatternKind, attribute_source_offset}`, `RootNode` | Memoized L1/L2 artifacts in `vize_incremental` (`SurfaceArtifact`, L2 page), and L1 entity decoding (`vize_l1` embed source) | **#6850/#6872** (Maestro reads incremental level artifacts) |
| `vize_maestro -> vize_relief`        | 24 files / 33 sites (ide/completion 6, hover 3, references 2, virtual_code 4) | `BindingType` (15), `RootNode`, `SourceLocation`, `TextCallContent`, `ErrorCode`                                                                                          | `BindingType`: L2 `SetupBindingKind` (#6844 S3). AST: L2 artifacts                                                                                                     | #6850/#6872 + #6844 S3                                                                     |
| `vize_maestro -> vize_croquis`       | 49 files / 119 sites (ide/completion 9, definition 6, hover 5, template_scope 4, diagnostics 4, virtual_code 4, rename 3) | `facts::{Demand, FactGroup, FactConsumer}` (re-exports of `vize_l0::fact`), `facts::{Bindings, BindingsTable, component_usage_list, reactivity_*}`, `Drawer`/`DrawerOptions`, `Analyzer`, `script_parser::{parse_program_for_analysis, parse_script_setup}`, `MacroTracker`, `ReactiveKind`, `ScopeKind`, `BindingOccurrences`, `naming::to_pascal_case` | `vize_l0::fact` (path swap for the fact-manager types); L2 setup facts; L2 references/`file::query` | **#6844** S8 (Maestro) for the facts; #6850 for the template scopes |
| `vize_maestro -> vize_atelier_sfc`   | 42 files / 73 sites (server/document_structure 5, server/state 4, ide/diagnostics 4, virtual_code 3, definition 3) | `SfcDescriptor` (33), `parse_sfc` (15), the `croquis` bridge (20), `script::*`, `prepare_root_patterned_template`, `validate_script_setup_semantics_located`, `BlockLocation`, `SfcCustomBlock` | `vize_incremental::SharedDescriptor` already serves memoized descriptors, but it is still the Croquis `SfcDescriptor`; L1 `AdmittedDescriptor` | Descriptor decision + #6850 + #6844 |
| `vize_maestro -> vize_atelier_jsx`   | 7 files / 10 sites (ide/jsx 6, diagnostics 1)                    | `lower_source`, `JsxLang`, `Severity`, `StyleExprSpan`, `ScopedStyle`, `JsxDiagnostic`                                                                                                                       | Shared L1->L2 JSX lowering                                                                                                                                           | #6885 step 9                                                                               |
| `vize_maestro -> vize_croquis_cf`    | 2 files / 4 sites (`ide/ecosystem/router.rs`, `router_project.rs`) | `providers::vue_router::{NamedRoute, RouteParams, RouteParam, parse_route_params}`, `providers::{PROJECT_FACTS, ProjectSources}` (route-name completion)                                                   | None. Project-level facts on the L2 query API over the shared project model                                                                                             | #6871 + #6874 (#6844 S9)                                                                    |
| `vize_incremental -> vize_relief`    | 1 file / 1 site (`artifact.rs`)                                  | `ErrorCode`, which is a re-export of `vize_l0::compiler_error::ErrorCode`                                                                                                                                        | `vize_l0::compiler_error::ErrorCode` (the same type)                                                                                                                  | **None. Slice 1 (#13363, closed: see the measurement note)**                                                                |
| `vize_incremental -> vize_croquis`   | 2 files / 2 sites (`descriptor.rs`, `artifact.rs`)               | `sfc::{SfcDescriptor, SfcError, SfcParseOptions, parse_sfc}`. Maestro reads `SharedDescriptor` as a Croquis `SfcDescriptor` everywhere                                                                 | `croquis::sfc::parse_sfc` is already a thin policy adapter over `vize_l1::container::vue::boundary::split_blocks`; L1 `AdmittedDescriptor` refuses custom blocks | Descriptor decision (below) |
| `vize_musea -> vize_croquis`         | 3 files / 4 sites                                               | `sfc::parse_sfc_without_css_vars` (Art block split; `<art>` is a custom block), `script_parser::parse_define_art` -> `macros::ArtDefinition` (title, description, category, status, order, tags, component), `script_parser::parse_script_setup` (import ranges, `defineArt` call span, binding names) | `SetupFacts::macro_calls` (kind `DefineArt` with spans), `SetupFacts::bindings`, L2 `Import` rows. No `defineArt` option-object fact. L1 descriptor refuses `<art>` | #6844 (a `defineArt` contract fact family in `vize_l2::framework::vue`) + descriptor decision |

## What is blocked on `levels.script` (#6844)

- `vize_patina -> vize_croquis` (S8; also S3 and S6, and family 11 via #6836/#6871).
- `vize_canon -> vize_croquis` (S10). Also behind #6849.
- `vize_maestro -> vize_croquis` (S8 for Maestro).
- `vize_musea -> vize_croquis` (macro facts for `defineArt`, bindings, imports).
- `vize_maestro -> vize_croquis_cf` (S9, project facts).
- The `vize_atelier_sfc::croquis` bridge parts of every `-> vize_atelier_sfc`
  edge (Patina, Canon, Maestro).
- `BindingType` uses inside the `-> vize_relief` edges of Maestro (S3).

The producers S1, S2, S3, S6 (partial) and S7 have landed in
`vize_l2::framework::vue`. **No product consumer has switched yet.** The 6844
record requires each switch to be byte-identical and to delete the Croquis read.

## Blocked on other gates

- **#6848** (Patina rules on L1/L2): `vize_patina -> {relief, armature}`.
- **#6849** (L4 TS projection default switch): all 6 Canon edges, and with them
  6 Patina and 6 Maestro rows.
- **#6850/#6872** (Maestro on incremental artifacts): `vize_maestro -> {armature, relief}`
  and the template-scope part of `-> croquis`.
- **#6885** step 9 (JSX): `vize_{patina,canon,maestro} -> vize_atelier_jsx`.
- **SFC descriptor placement** (no issue yet): the container part of every
  `-> vize_atelier_sfc` edge, plus `vize_incremental -> vize_croquis` and the
  block-split part of `vize_musea -> vize_croquis`.

## Edges removable without a route switch

1. `vize_incremental -> vize_relief`: a path swap to the same L0 type
   (slice 1, #13363).
2. Hygiene that removes no edge yet, but shrinks the later diffs:
   - Canon `vize_atelier_core::TemplateSyntaxMode` -> `vize_l0::TemplateSyntaxMode` (10 files).
   - Maestro `vize_croquis::facts::{Demand, FactGroup, FactConsumer}` -> `vize_l0::fact::*`.
   These were left out on purpose: alone they change no allowlist row. Fold
   them into the first switch PR of each crate.

No other edge can be removed by moving imports. Every other site reads a
legacy AST, legacy facts, or the legacy descriptor. So each further slice is
either a producer PR at a level crate (#6844, #6848, #6885) or a default-off
route switch.

## Suggested order (after slice 1)

1. **Descriptor decision**, then `vize_incremental -> vize_croquis` and the
   Musea block split. If the compatibility descriptor moves under
   `vize_l1::container::vue`, the output is unchanged by construction: it is
   already a thin policy adapter over `boundary::split_blocks`. The
   `SfcScriptBlock::bindings` field must be split off first (see decision 1).
   The move also unblocks the container parts of the Patina, Maestro and
   Canon `-> atelier_sfc` edges. If the descriptor does not move, the L1
   descriptor must admit custom blocks first.
2. **Musea `defineArt` facts** (#6844 family 3/4 extension): an
   `ArtContract` beside `ComponentContract`, with a Croquis-equality
   differential on the musea corpus. Then switch Musea, and
   `vize_musea -> vize_croquis` is gone (-1 Maestro row).
3. **#6885 step 9** (JSX owner): drops `-> vize_atelier_jsx` from Patina and
   Maestro (-3 rows). Canon's JSX edge waits for 6849 step 3.
4. **#6844 S8** for Patina, one rule family per PR, then the #6848 rule moves.
   Patina's direct edges go last, all together, because every rule must leave
   the `Rule` Relief hooks.
5. **#6849** SFC default switch, then delete `virtual_ts` (#6828). That
   removes 18 rows at once, but it is the longest track.
6. **#6850/#6872** for Maestro's per-request re-parses (28 files).

## Decisions for the maintainer

1. **Where the compatibility `SfcDescriptor` lives.** Today it is
   `vize_croquis::sfc`, an L1-backed adapter that is still a legacy crate.
   Every product reads it (Croquis inventory family 13: 179 / 275 / 103 sites).
   There are two options:
   - (a) Move it to `vize_l1::container::vue::compat`, keeping
     `BindingMetadata` and the Relief `From` impl in Croquis.
   - (b) Require every consumer to move to `AdmittedDescriptor`. Custom blocks
     (`<art>`, `<i18n>`, `<docs>`) would then have to be admitted first.

   (a) is byte-identical and fast. (b) matches "container is the file-format
   layer" more strictly, but it is a route switch for every product.

   (a) has three costs:
   - `SfcScriptBlock::bindings` is a Relief `BindingMetadata`. The compiler
     fills it in `compile_script/{inline,function_mode}`, and it is part of
     the serde shape. It would have to leave the block type, for example as a
     side table in `vize_atelier_sfc`.
   - The `v-bind()` CSS variable scan (`css_transform`, 600 lines) has to come
     along or stay behind a hook.
   - `vize_l1` gains `serde` and `is_void_tag`.
2. **Canon's `legacy` feature** enables `vize_armature/legacy` for the
   `LegacyVueVersion` model. An optional edge still counts. The version model
   needs a level home (`vize_l0::config`/dialect profiles), or the feature
   needs to move out of the product.

## Measurement note: dependency-graph-only changes and ci-opt size

Slice 1 (#13363, branch `refactor/incremental-l0-error-code`) is a pure
import-path swap. Instruction rows: 107/107 identical, A/A 0. Wall and RSS are
stable. But the ci-opt normal CLI grew by +32,512 B (+0.040%).

A probe (run 44461337780) compared the two profiles:

- **Fat LTO, 1 CGU (release, shipped):** every section and the stripped size
  are identical (66,916,176 B). The net symbol delta is 0.
- **Thin LTO, 16 CGUs (ci-opt):** 26,265 of 122,972 symbols change size. Most
  of the change is in `anon.*.llvm.*` partition names.

The cause is that dropping any normal edge changes `-C metadata`, and with it
the StableCrateId, of the crate and of everything downstream of it. Thin LTO
partitions by those hashes. **Every remaining slice that edits the dependency
graph of Maestro, Canon or Patina will shift the ci-opt size**, in either
direction, with no code change. This needs a maintainer rule before any edge
removal can auto-merge. One option is to judge dependency-graph-only slices on
release-profile size, or with a thin-LTO repartition A/A band.

## Status at the 2026-11-19 pause

- **#13375 merged** (`vize_incremental -> vize_relief`). Maestro's allowlist is now 19 on main.
- Maintainer rulings:
  - Binary size is judged on the shipped release profile (fat LTO, 1 CGU), which must not grow.
    Instruction, wall, CPU and RSS stay strict.
  - `SfcDescriptor` moves to `vize_l1::container::vue`.
  - Canon's legacy version model goes into a level crate. It was already in `vize_l1::dialect`:
    Canon now re-exports it from there, on branch `refactor/sfc-descriptor-l1`.
- Branch `refactor/sfc-descriptor-l1` (no PR yet) moves the descriptor and drops
  `vize_incremental -> vize_croquis`. Its state, its in-flight probe 44492961984 and its next steps
  are in `.handoff/refactor-sfc-descriptor-l1.md` on that branch.
- Next after that branch: the Musea `defineArt` fact in a new file under
  `vize_l2/src/framework/vue/` (lane A owns the directory), then drop `vize_musea -> vize_croquis`.
- Measurement helpers: `iccmp.py` and `cgdiff.py` (callgrind self-cost diff) are committed beside
  this file under `.handoff/tools/`.
