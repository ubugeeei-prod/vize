# Native script analysis: Croquis inventory and migration order

Related: #6844 (gate `levels.script`), #6851 (`products.dependencies`), #6828
(legacy deletion), #6836, #6837, #6871, #6874.

The maintainer decision on #6844 stands: no adapter presents Croquis output as
Davinci facts. Script analysis is built natively, from the L1 script parse to
L2 script facts, and anything unfinished is reported as unfinished. This record
inventories every Croquis output that a consumer reads, says which native piece
already exists for it, and fixes the order in which consumers move. The counts
are the symbol-resolved sites printed by
`node tools/support/compat/davinci/croquis-consumers.ts --summary` at
`cdec750761`; re-run it for current numbers.

## Decisions

1. **Compiler first.** The first consumer to move is the `vize_atelier_sfc`
   script-setup route, because every step there has byte-parity oracles: the
   compiler snapshots, the Vue differential corpus and the real-project
   reductions under `tests/_fixtures/differential/compiler`. Patina and Maestro
   follow, Cross-file analysis (`vize_croquis_cf`) after the per-file facts it
   reads, and Canon last (its projection emitters and content mapper have a
   separate owner).
2. **One fact family per PR.** Each native producer PR adds one family and a
   differential test that compares it with Croquis on a corpus with exact
   equality. Croquis is a dev-dependency of that test only. It is never a
   runtime fallback, and a native refusal stays a refusal. Where Croquis
   disagrees with Vue, Vue is the authority and the row is recorded as a
   Croquis defect, not copied.
3. **One walk.** Script facts come from the existing single L2 Program walk
   (`vize_l2::lang::js::file`), which already records scopes, declarations,
   references, imports and exports. Vue meaning is added by a `FileObserver`
   that runs inside that walk; no second traversal or parse is added. This
   follows the 2026-09-27 design comment on #6844 ("when a product needs more,
   the same walk is extended").
4. **Placement.** Vue script facts (macro table, macro calls, binding kinds,
   props/emits/model/slots contract, reactivity facts) live in a new
   `vize_l2::framework::vue` module, as the 2026-09-28 comment on #6844 already
   assigned the binding-kind table. They do not add public types to
   `vize_l2::artifact`, which #6838 is reshaping. The callee-name macro table is
   a `const` table there: L1→L2 depends on L2, and L3, L4 and the products read
   the same macro kinds.
5. **Consumer switches are byte-identical.** A switch replaces a Croquis read
   with the native fact and proves identical output on the oracles above in the
   same PR. The switch deletes the Croquis edge it removes; the generated
   consumption ledgers and the product dependency allowlists only shrink.

## Inventory

Sites are resolved product sites (`Croquis.<field>` and product types) plus the
non-product imports named in the row. The level crates show 13 `bindings` and
`scopes` field sites; they are resolver false positives (no level crate source
mentions Croquis), and are excluded below.

| #   | Fact family                    | Croquis producer (production lines)                                                  | Read through                                                                                                                                                       | Consumers (sites)                                                                                                         | Native today                                                                                                                                                                                                                                                                     |
| --- | ------------------------------ | ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Script parse and orchestration | `script_parser` parse/result (0.6k); `vize_atelier_sfc::croquis` bridge (2.3k)       | `parse_program_for_analysis`, `parse_script_setup*`, `ScriptParseResult`, `analyze_sfc_descriptor*`, `SfcScriptParses`, `Drawer`, `DrawerOptions`                  | atelier_sfc, canon, patina, maestro, vitrine, jsx                                                                         | L1 `parse_program_once` (one OXC parse, retained AST, exact spans); container `ScriptView`/`ScriptRole`; L2 `ProgramInput`/`FileProducer`                                                                                                                                        |
| 2   | Setup binding kinds            | `script_parser/process` (4.2k), `value_binding_origins` (0.6k)                       | `Croquis.bindings` (79), `Croquis.binding_spans` (10), `BindingMetadata` (38), `BindingType` (226, from `vize_relief`)                                             | atelier_sfc 25+37, atelier_core 10, vapor 12, jsx 12, canon 7, patina 5, dom 2, maestro 2, vitrine 2, ssr 1, croquis_cf 1 | L2 neutral `Declaration` rows (const/let/var/import/function/parameter/class, primitive initializers); `VueExposure` (direct let/const/var membership); `vize_l1_to_l2::BindingKind`/`BindingTable` (L2 DOM lane input, still built from legacy metadata). No Vue classification |
| 3   | Macro calls and table          | `macros` (1.2k), `script_parser/extract` macros (0.3k)                               | `Croquis.macros` (205), `MacroTracker`, `MacroKind`, `MacroCall`, `DEFINE_*`, `is_builtin_macro`, `is_runtime_erased_macro`, `macro_artifact_kind`                 | canon 109, croquis_cf 30, patina 24, maestro 21, atelier_sfc 18+16, vitrine 3                                             | None. The native setup policy refuses every call                                                                                                                                                                                                                                 |
| 4   | Props/emits/model/slots/expose | `script_parser/extract` props, emits, expose (1.0k); `macros/*`                      | `PropDefinition` (38), `EmitDefinition`, `ModelDefinition`, `PropsDestructuredBindings`, `Croquis.macros` accessors                                                | canon, patina, croquis_cf, atelier_sfc `ScriptCompileContext`                                                             | None                                                                                                                                                                                                                                                                             |
| 5   | Types                          | `types` (1.8k), `script_parser` type definitions                                     | `Croquis.types` (39), `Croquis.type_exports` (30), `ResolvedTypeWorld` (16), `TypeLookup` (26), `TypeModule` (10)                                                  | atelier_sfc 18, canon 12+27, patina 4, marquette 3, vitrine 2                                                             | L2 `Namespace::Type` declarations and type-only imports; primitive setup annotations. No macro type-argument or cross-file resolution                                                                                                                                            |
| 6   | Imports and exports            | `script_parser/process` imports, module exports                                      | `Croquis.import_statements` (9), `invalid_exports` (5), `re_exports`, `re_export_forwards`, `AuthoredModuleExport` (9)                                             | canon 6, atelier_sfc 2+4, patina 1, croquis_cf, vitrine                                                                   | L2 `Import`/`Export` rows with module source sites; `OriginalModuleSources`                                                                                                                                                                                                      |
| 7   | Reactivity                     | `reactivity.rs`, `reactivity_tracking` (1.5k)                                        | `Croquis.reactivity` (9), `ReactiveKind` (102), `ReactivityLossKind` (73), `reactivity_*` helpers                                                                  | atelier_dom 4 (inline `.value`/`_unref` reads), atelier_core, canon, croquis_cf, vitrine                                  | None. The DOM lane projects Croquis into `ReactiveRead` (`vize_atelier_dom/src/compile/croquis_facts.rs`)                                                                                                                                                                        |
| 8   | Unused bindings, occurrences   | `script_parser/unused`, `binding_occurrences`, `facts::spec`                         | `Croquis.unused_bindings` (21), `BindingOccurrences` (16), `UnusedBindings` (9)                                                                                    | atelier_sfc 13 (lint/LSP bridge), patina 7, vitrine 1                                                                     | L2 references with resolved/unresolved targets; `file::query`                                                                                                                                                                                                                    |
| 9   | Setup context and flow         | `setup_context` (0.8k), `provide.rs`, `race.rs`, `effect_graph`, `call_graph` (1.5k) | `Croquis.setup_context` (5), `ProvideKey` (60), `InjectPattern` (27), `EffectGraphSummary` (35), `RaceConditionRisk*`                                              | croquis_cf, patina, vitrine, curator, `vize` CLI, doctor                                                                  | None                                                                                                                                                                                                                                                                             |
| 10  | Options API and Nuxt           | `script_parser/process` options API, class components, Nuxt render (1.6k)            | `Croquis.options_descriptor` (4), `OptionGroup` (15), `nuxt_render_template` (10), `nuxt_static_render_script` (8), `component_shape`, `component_registrations`   | `vize` CLI 16, patina 9, canon 10, croquis_cf 3, maestro 2                                                                | L2 `OrdinaryEmptyScript` (empty `export default {}` only)                                                                                                                                                                                                                        |
| 11  | Template-facing facts          | `drawer` (11.1k), `scope` (2.6k)                                                     | `Croquis.scopes` (151), `ScopeKind` (229), `template_expressions` (31), `ComponentUsage` (138), `PassedProp` (73), `template_info` (23), `element_ids`, directives | canon, croquis_cf, maestro, patina                                                                                        | Not script analysis. Owned by L2 template resolution and the query API (#6836, #6838, #6871); listed so #6844 does not claim it                                                                                                                                                  |
| 12  | Facades and snapshots          | `analyzer`, `croquis` (3.9k), `facts` (3.2k)                                         | `Analyzer` (313), `AnalyzerOptions` (288), `Croquis` (618), `CroquisFacts`, `FactConsumer`, `Demand`, `CroquisSemanticSnapshot`                                    | canon 260+256+331, croquis_cf 119, patina 42, sfc 43, jsx 29, core 18, maestro 18                                         | Entry points; each disappears with the last switch of its consumer                                                                                                                                                                                                               |
| 13  | Non-script surfaces            | `sfc` (1.2k)                                                                         | `SfcDescriptor` (179), `SfcParseOptions` (275), `SfcError` (103); CSS `extract_and_transform_v_bind*`, `extract_css_vars`                                          | atelier_sfc and every SFC consumer                                                                                        | Out of #6844 scope: the container gate (`levels.container`, #6837) and the style lane                                                                                                                                                                                            |

`vize_croquis_cf` is the cross-file analyzer (31.1k lines: rules 12.0k,
analyzer 9.7k, providers 3.3k, facts 2.0k), not a control-flow analysis. It is
itself a Croquis consumer (461 product sites, mostly families 2, 3, 9 and 11)
and is read by the `vize` CLI, `vize_curator`, `vize_vitrine`, `vize_doctor`
(optional) and `vize_maestro`. Its native replacement is project-level facts on
the L2 query API (#6871) over the shared project model (#6874), after the
per-file families it reads exist.

The compiler also runs a second legacy script analysis of its own:
`ScriptCompileContext` (`vize_atelier_sfc/src/script/context`), an OXC walk
that produces `BindingMetadata`, macro spans and type resolution and uses
Croquis types (`PropDefinition`, `EmitDefinition`, `ResolvedTypeWorld`) as
data. Families 3 to 5 replace it, together with L4 span-level setup rewriting.

## Migration order

Each slice is one PR, or a producer PR and a switch PR. A slice that cannot be
finished stays open on #6844 with its refusals listed.

| Slice | Native producer                                                                                                                                                       | Consumer switch and oracle                                                                                                                                                                                                                       |
| ----- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| S1    | `framework::vue` macro table: callee name → kind, lifecycle, artifact kind                                                                                            | Compiler macro-name lookups (`compile_script/macros.rs`, `statement_sections.rs`, `artifacts*.rs`, `script/define_*.rs`, `script/context/helpers.rs`). Oracle: table equality with Croquis, compiler snapshots unchanged                         |
| S2    | Top-level macro call facts from the walk observer: kind, call and callee spans, declarator pattern, type-argument and runtime-argument spans, `withDefaults` wrapping | None yet. Oracle: `MacroTracker::all_calls()` on the corpus                                                                                                                                                                                      |
| S3    | Setup binding kinds: ref/maybe-ref/reactive-const/const/let/literal-const/props/props-aliased, destructuring, enums, classes, imports                                 | None yet. Oracle: `Croquis.bindings` (names, kinds, props aliases) on the corpus                                                                                                                                                                 |
| S4    | Reactive read facts per setup binding                                                                                                                                 | Compiler setup route reads S3 and S4 instead of `analyze_script_setup_program_to_summary` and `croquis_to_legacy_bindings`; the DOM projection and `atelier_core` lane context read S4. Oracle: compiler snapshots, Vue differential, reductions |
| S5    | Component registration facts                                                                                                                                          | `is_component_registered` in the template lanes                                                                                                                                                                                                  |
| S6    | Props/emits/model/slots/expose contract; macro type arguments through lazy project-model type resolution (#6874)                                                      | `ScriptCompileContext` macro processing; L4 span-level setup rewriting expands the native SFC route                                                                                                                                              |
| S7    | Normal `<script>` bindings and the Options API descriptor                                                                                                             | `collect_normal_script_bindings`, `collect_options_api_bindings`                                                                                                                                                                                 |
| S8    | Families 6, 8 and 10 for Patina, then Maestro                                                                                                                         | Patina rules and Maestro features, one rule family or feature at a time; product allowlists shrink                                                                                                                                               |
| S9    | Family 9 and project-level facts                                                                                                                                      | `vize_croquis_cf`, vitrine, curator, CLI, doctor                                                                                                                                                                                                 |
| S10   | Remaining families for Canon                                                                                                                                          | Canon consumers, coordinated with the owner of the projection emitters and content mapper                                                                                                                                                        |

The gate passes when no product reads Croquis script analysis: every family
above except 11 and 13 has a native producer and no consumer imports its
Croquis form. Family 11 closes with #6836/#6871 and family 13 with #6837 and
the style lane; `vize_croquis` itself is deleted under #6828.
