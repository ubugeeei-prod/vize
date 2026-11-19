# Handoff: S6c core, first slice — the compiler's macro list from `SetupFacts` (#6844)

Delete this file before merge. No code yet; this records the analysis.

## Where the legacy macro list is built

- `crates/vize_atelier_sfc/src/compile.rs` (`compile_script_setup` path) already parses the
  setup once with L1 (`setup_facts::SetupAnalysis::new`). It then calls
  `ctx.analyze_for_target(setup.program(), ..)`. That reuses the retained L1 Program, so on this
  path there is **no second parse**. The perf gain of this slice is the removed AST walk, the
  `String` copies of every macro's argument and type-argument text, and the duplicate macro
  table — not a parse.
- `ScriptCompileContext::analyze()` (a fresh OXC parse via `parse_with_oxc`) is still used by
  `script::{analyze_script_setup, extract_macros}`, by `compile_script/inline/compiler.rs`
  `build_script_setup_context`, and by `compile_script/props/validation.rs`. These are
  separate routes; check which production entries still reach them before counting a parse as
  removed.
- Macro registration is interleaved with binding registration in one walk:
  `script/context/parse.rs` `process_statement` and `register_macro`, plus the helpers in
  `script/context/helpers.rs`: `extract_macro_from_expr`, `extract_args_from_call`,
  `extract_type_args_from_call` and `macro_binding_name`.

## Exact legacy semantics a native list must reproduce (or fix with pinned fixtures)

| Legacy behavior                                                                                                                                | Native `framework::vue::MacroCall`                                            | Gap                                                                                                                  |
| ---------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| Only a direct `CallExpression` init/statement; no TS-wrapper or paren unwrap                                                                   | unwraps `as`/`satisfies`/`!`/`<T>`/parens like Vue                            | `const p = defineProps() as X` is a macro natively, not in legacy → Vue is the authority; pin a fixture + ledger row |
| `is_builtin_macro` names only                                                                                                                  | table includes ecosystem macros (definePage, …)                               | filter `Macro::builtin`                                                                                              |
| `type_args` = text inside `<…>`                                                                                                                | `type_arguments` span includes the angle brackets                             | strip one `<`/`>`                                                                                                    |
| `args` = first argument start .. last argument end                                                                                             | `arguments`: same definition                                                  | none expected; verify empty-args `""`                                                                                |
| `binding_name` = identifier, or first element of an array pattern (`macro_binding_name`)                                                       | `declarator` = whole pattern span                                             | derive the name from the pattern text the same way                                                                   |
| `withDefaults`: `with_defaults.args` = **whole call text**, `type_args = None`; inner `defineProps` registered with the wrapper's binding name | records `WithDefaults` + inner `DefineProps` (declarator = wrapper's pattern) | map exactly                                                                                                          |
| Duplicate calls: `Option` fields keep the **last** call; `define_models` keeps all                                                             | records every call; the contract's `duplicates` lists later ones              | take the last per kind for the `Option` fields                                                                       |
| Object-pattern `defineProps` destructure registers props + `props_destructure`                                                                 | `PropsDestructure` / `unclassified`                                           | keep the legacy `process_props_destructure` until bindings move (S4b)                                                |

## Proposed slice

1. Add `SetupAnalysis::macros(&self, content) -> ScriptSetupMacros` in
   `compile/setup_facts.rs`. It builds the legacy `MacroCall`s from `SetupFacts::macro_calls()`
   with the table above.
2. In `process_program`, stop calling `register_macro` and stop building `with_defaults` /
   `define_props` `MacroCall`s on the `compile.rs` path. Keep binding and props-key extraction.
   Assign `ctx.macros` from step 1 before `validate_macro_scope_for_descriptor`.
3. Where Vue and legacy disagree (TS-wrapped macro), follow Vue: add a pinned fixture under
   `tests/_fixtures/differential/compiler` and a ledger row.
4. Proof: byte-identical `vize build` DOM/SSR/Vapor on the repo corpus, base versus head. Also
   the Vue differential corpus, `crates/vize_atelier_sfc/tests/davinci_production_reach.rs` floors
   (DOM 347, SSR 349, Vapor 203 of 351), release-profile instruction rows (`atelier_*`,
   `sfc_*`) ≤ base, and a same-size release binary.
