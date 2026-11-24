# vize house style

Draft for the maintainer's review. It covers every first-party language: Rust, TypeScript and JavaScript, Vue SFCs, Pkl, shell, Nix, editor scripts (Lua, Vim script, Emacs Lisp) and the code inside docs.
Examples come from `origin/main` at `0a04b42431` (2026-11-24). Paths are repo-relative and given as `path:line`.

Each rule has an **evidence class**, which says what a PR that applies the rule to existing code must prove (the details are in `plan.md` §3):

| Class | Meaning | Rust proof | TS/JS proof |
| --- | --- | --- | --- |
| **N0 neutral text** | Only comments and whitespace change | Token identity (non-doc), plus IR identity with panic locations normalized, plus an identical release size | AST identity (oxc-parser, comments and positions dropped) |
| **N1 move/rename** | Items move or get new names; no logic changes | The existing `ir-identity.ts` with relocations, plus byte-identical output and no release growth | AST identity after the rename map, plus identical built `dist/` (comments stripped) |
| **N2 value-preserving** | A literal becomes a named `const`, or a list becomes a table with the same values | IR identity (constants fold at MIR) | Identical tests and built output |
| **C code change** | Anything else | Full fleet evidence: instruction counts, PR performance observation, parity | Tests and bundle-size check |

Rules that would need class C on hot paths only apply to **new code** and to code that is already being rewritten for another reason.

---

## 0. Principles

1. **A reader opens a file and knows where they are.** The file name says what it holds, the first lines say why it exists, and the order goes from public to private and from entry to helpers.
2. **Comments say why, never what or when.** The code says what. Git says when and who. Tests and ledgers say which issue.
3. **Data lives in tables, not in control flow.** Lists of names, versions, paths, limits and help text belong in a `const` table, a data file or config, with one owner.
4. **Whitespace marks paragraphs.** One blank line separates items, and inside a function one blank line separates steps. Never two blank lines, and never a wall of 80 lines without one.
5. **Formatters decide everything they can.** This guide only adds what `rustfmt`, `oxfmt` and `pkl format` cannot decide. Each such rule is checked by a ratchet or a custom lint (§8).
6. **Never rewrite pinned bytes by hand.** Generated files are edited at their source. Fixtures and snapshots are oracles. Byte-custody sources go through the custody procedure (`plan.md` §4).

---

## 1. Rules for every language

### 1.1 File size and splitting

| Rule | Limit | Enforced by |
| --- | --- | --- |
| New files | ≤ 500 lines | `source-file-lengths.ts` (exists) |
| Existing files over 500 lines | may never grow, and must shrink when touched | `source-file-lengths.ts` (exists) |
| `tools/commands/**` and `tools/support/**` | the same 500 lines. **These are exempt today** (`source-file-lengths.ts:58-59`), and the exemption is dropped after the tools move | `source-file-lengths.ts` |
| A function or method | aim for ≤ 60 lines; 120 is the hard limit for new code; existing functions over 120 lines may not grow | new `function-lengths` ratchet (§8) |
| A test function | ≤ 80 lines, with setup moved into a fixture or builder | same ratchet |

How to split a file:

- **Split by responsibility, never by size alone** (CONTRIBUTING already says this). A good split leaves a parent module whose body reads like a table of contents.
- The parent keeps the public entry and the types a caller needs. Each child holds one concern and is named for that concern. A child is never named `part2`, `more`, `helpers2`, `_ext` or `misc`.
- Test code leaves the file first (§1.6). It is the cheapest split: it is test-only, and the `vize` binary is unchanged.
- Data leaves the file second (§1.4). A 300-line `match` that maps kinds to text is a table.
- Do not shard one concept across many tiny files a reader has to hop through. A file under about 40 lines that has a single caller usually belongs to that caller.

Example, `crates/vize/tests/check_cli.rs` (3,750 lines, 15 functions over 80 lines):

```text
before: crates/vize/tests/check_cli.rs               3,750 lines: json output, tsconfig inputs, nuxt, stubs, corsa discovery ...
after:  crates/vize/tests/check_cli.rs               ~60 lines: `mod` list + shared `create_cli_project`
        crates/vize/tests/check_cli/json_output.rs
        crates/vize/tests/check_cli/tsconfig_inputs.rs
        crates/vize/tests/check_cli/nuxt.rs   ...    each ≤ 500 lines, named by the behaviour under test
```

### 1.2 Naming

**Applies everywhere:**

- **No issue or PR numbers, dates or version numbers in names**: not in file names, directory names, functions, tests, types or workflow subjects. Today there are **442 files** with an issue number in the name. The worst areas are `crates/vize_canon` (242) and `tests/tooling` (74). Examples:
  - `crates/vize_patina/tests/no_dupe_keys_447.rs` → `crates/vize_patina/tests/no_dupe_keys/computed_and_data.rs` (name the behaviour)
  - `crates/vize_maestro/src/ide/semantic_tokens/regression_8014_tests.rs` → `semantic_tokens/tests/<behaviour>.rs`
  - `tests/tooling/native-attribute-values-7502-hook.test.ts` → `tests/tooling/native/attribute-values-hook.test.ts`

  The issue number lives in exactly one place: the fixture manifest's `issue` field (or the ledger row), which is data.
- **No codenames** outside `docs/davinci/` (fleet rule 8). The scripted rename owns this; readability lanes do not add codenames and do not rename them on their own.
- **No generic names**: no `utils`, `helpers`, `common`, `shared`, `misc`, `stuff`, `new`, `tmp` and no numbered siblings. There are 54 such files today (24 `helpers.rs`, 7 `utils.ts`). Name them by what they hold: `helpers.rs` → `identifier_scan.rs`.
- **Names are words.** Abbreviate only well-known terms (`ast`, `ir`, `css`, `sfc`, `lsp`, `uri`, `ctx`, `id`). Spell out everything else: `prev`/`cur` are fine as loop locals, but not as fields.
- **No combinatorial entry names.** An options struct replaces `_with_x_and_y_and_z` suffixes. Example: `crates/vize_atelier_dom/src/compile/custom_elements.rs:116`
  `compile_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options(…)` (116 characters), plus 11 siblings in the same file, and `davinci/vize_l4/src/framework/vue/dom/budget/slot_scope.rs:62` `emit_dom_source_observed_with_options_captured_slot_scope_environment_and_syntax`
  → `compile_template(source, &CompileOptions { custom_elements, template_syntax, hoisted_scope_id, sections, codegen })`.
  That change is class C (it changes signatures). It is owned by the `L1Ir`…`L4Ir` entry-point track (the maintainer's level-IR naming decision of 2026-11-24); readability lanes only stop new ones (§8).
- **Length budget**: ≤ 40 characters for a product function, ≤ 70 for a test name, ≤ 48 for a file stem. Today 245 public Rust functions are over 40 characters and 8,757 of the 24,410 Rust tests are over 60.

**By language:**

| Thing | Rust | TS / JS | Vue | Pkl | Shell |
| --- | --- | --- | --- | --- | --- |
| Files | `snake_case.rs` | `kebab-case.ts`, including composables (`use-mouse.ts`, as `npm/compose` already does) | `kebab-case.vue` (as `npm/ui` does; component *names* are PascalCase in code) | `PascalCase.pkl` for modules other files import, `kebab-case.pkl` for leaves | `kebab-case.sh` |
| Types | `UpperCamelCase`; acronyms as words (`SfcDescriptor`, not `SFCDescriptor`) | `PascalCase`, same acronym rule | — | `PascalCase` classes | — |
| Functions | `snake_case`, verb first for actions (`collect_bindings`), noun for pure getters (`span`) | `camelCase`, `function` declarations at module level (17,992 today, against 2,163 `const` arrows) | — | `camelCase` | `snake_case` |
| Constants | `SCREAMING_SNAKE` | `camelCase` for module constants (already 13,479 against 1,851); `SCREAMING_SNAKE` only for environment-variable names and wire constants | — | `camelCase`, with `fixed`/`const` | `SCREAMING_SNAKE` only for exported env vars |
| Tests | `fn <subject>_<behaviour>()`, read as a sentence; the module path carries the subject | `test("<subject> <behaviour>")` | — | — | — |

Outliers to fix: 222 TS files that are not kebab-case (`npm/fresco` 73, `playground/src` 61, `npm/builder` 39, `tools/support` 33). For example `npm/fresco/src/hitTesting.ts` → `hit-testing.ts` and `playground/src/features/canon/sourceMappings.ts` → `source-mappings.ts`. Also 23 non-snake Rust files (`tools/support/compat/parser/frozen-10682-observer.rs`).

Renames are class N1. They have two traps (`plan.md` §3.3):

1. `#[derive(Debug)]` and serde put type and field names into strings. A rename changes those bytes and can change output.
2. Package files may be deep-import paths that users depend on.

### 1.3 Comments

**Delete:**

| Kind | Today (heuristic, §inventory) | Example → after |
| --- | --- | --- |
| **Issue-number archaeology** | 2,667 lines | `crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs:114-115` "…where v-text / v-html are as safe as on the literal tag (#3211)." → drop "(#3211)". The sentence already says why. `npm/builder/vite/src/native-shape-binding.test.ts:67` "`#11764: the real N-API producer and builder must share comment geometry.`" → "`The real N-API producer and builder must share comment geometry.`" |
| **History narration** ("used to", "previously", "no longer", "after the fix", "for now") | 834 lines (about 50% precision) | `crates/vize_patina/src/visitor.rs:215-216` "Derive the element's cheap facts in one property pass. They used to rescan the opening tag independently before every callback batch." → "Derive the element's cheap facts in one property pass, so callback batches never rescan the opening tag." (State the invariant, not the past.) `crates/vize/src/commands/lint.rs:377-378` "…a successful return used to leave `.vize/patina/session-*` behind." → "…otherwise a successful return leaves `.vize/patina/session-*` behind." |
| **Restating the code** | 2,010 lines (about 75% precision) | `crates/vize_croquis/src/import_resolver.rs:192` `// Create cache key` above `let cache_key = …` → delete. `:195` `// Check cache first` above `if let Some(cached) = self.cache.get(…)` → delete. `crates/vize_fresco/src/render/tree.rs:82,87,98,103` `// Update render tree` / `// Update layout tree` → delete. If the two steps need naming, extract `fn link_render(...)` and `fn link_layout(...)` (class C). |
| **Restating doc comments** | part of the above | `crates/vize_canon/src/types.rs:17` `/// Create a new type info.` on `pub fn new` → delete. `npm/compose/core/src/use-usb.ts:27` `/** Serial number. */` on `serialNumber` → delete. Keep `:56` `` /** `bRequest`. */ ``: it maps the field to the spec name, which is information. |
| **Section banners** | 211 lines | `crates/vize/src/commands/ide.rs:127-129` `// ====… VSCode Extension ====…` → delete. If the file needs sections, it needs modules (`ide/vscode.rs`). |
| **Commented-out code** | 132 flagged (low precision; review by hand) | Delete it. Git keeps it. |
| **Narrated provenance in tests** | — | "Regression (#1065/#1090): …" (`crates/vize_armature/src/parser/tests.rs:1713`) → the test name says the behaviour, and the fixture manifest holds the issue. |

**Keep**, and write more of these:

- **Why**: a constraint the code cannot show. A good example is `.cargo/clippy.toml:17-25`, which explains why a `DashMap` guard across `.await` deadlocks the LSP. Drop its `(#3315, #3373, #3377)` and keep everything else.
- **Invariants and contracts**: "`cursor` never splits a UTF-8 sequence", or "callers pass sorted spans".
- **`// SAFETY:`** on every `unsafe` block (fleet rule 7). It is mandatory and never deleted.
- **Parity notes** that name the upstream behaviour being matched, with a file or spec reference and no issue number: "matches `@vue/compiler-sfc` `processDefineProps`".
- **Module headers** (`//!`, or the file's first JSDoc) that say what the file is for, in 1–3 sentences. Every file over 150 lines has one.
- Tool directives: `#[expect(…, reason = …)]`, `// oxlint-disable-next-line … -- reason`, `@ts-expect-error reason`.
- **Comments that are semantics, not prose.** Never delete these: `/* @__PURE__ */` and `/*#__PURE__*/` (tree-shaking), `/* @vite-ignore */`, `/* webpackIgnore: true */`, `//# sourceMappingURL=`, `// @ts-*`, `/// <reference …/>`, `#!` shebangs, license and provenance headers (`crates/vize_maestro/src/ide/style/data/*.rs:1-3`), `// SAFETY:`, and HTML comments inside Vue templates or test inputs (the compiler can emit them). The sweep codemods refuse to touch them.

**Form:**

- Full sentences, starting with a capital and ending with a period. Wrap at **100 columns**, the same as `max_width`. Today only 199 Rust and 91 TS comment lines are over 100, so there is no reflow churn: do not re-wrap comments you do not otherwise touch.
- A comment sits on the line above what it explains. No trailing comments after code, except short enum or field notes in tables.
- `TODO` is allowed only as `TODO(#1234): …` with an open issue; the ratchet tracks the count. This is the one place an issue number stays, because it is a live pointer, not history.
- **Doc comments that are product text are not comments.** On clap `Parser`/`Args`/`Subcommand`/`ValueEnum` derives (28 files), `///` becomes `vize --help` output. On `#[napi]` items (47 files) it becomes the shipped `index.d.ts`. On `#[wasm_bindgen]` items (12 files) it becomes JS docs. Editing them is a **behaviour change** (help snapshots, generated typings). Comment sweeps skip these items; a deliberate edit goes through the normal product PR.

### 1.4 Hardcoding

| Smell | Rule | Example → after | Class |
| --- | --- | --- | --- |
| **Magic numbers in logic** | Every limit, threshold, capacity factor or bit pattern that is not `0`, `1`, `2` or an obvious index gets a named `const` with a one-line why. Shared limits live in one module per crate (`limits.rs`), or in `vize_l0::limits` when two crates share them. | The recursion limit `128` is spelled inline in `crates/vize_croquis/src/types/world/props.rs:47` and `:143`, `emits.rs:34` and `crates/vize_glyph/src/native_doc/template.rs:123`. Elsewhere it is a named constant with different values: `MAX_DEPTH = 64` in two croquis files, `MAX_RESOLUTION_DEPTH = 256`. → `vize_croquis::limits::TYPE_EXPANSION_DEPTH: usize = 128; // deeper aliases are cyclic in practice; stop instead of overflowing the stack`. There are 64 inline depth comparisons and 36 separately named depth constants today. | N2 |
| **Inline lists that are really tables** | A list of names, globs, products or extensions with more than about 5 entries, or one read in more than one place, becomes a `const` table, data file or config next to its owner. Control flow then reads the table. | `tools/commands/ci/source-file-lengths.ts:51-95`: `isExcluded` hardcodes about 30 paths, globs and segments, and duplicates the ignore lists in `vite.config.ts`. → `tools/config/source-file-lengths.json` (`{ "excludedSegments": [...], "excludedFiles": [...] }`), or better, mark generated files once in `.gitattributes` with `linguist-generated` and read that. | N2 for tools |
| **Help or prose text built in code** | Long user-facing text is one `const` string per entry, or an `include_str!` of a Markdown file, not 30 `push_str` calls. | `crates/vize_croquis_cf/src/diagnostics/formatting.rs:38-460`: `format_kind_details` is 423 lines with no blank line, each arm pushing text line by line. → a `const KIND_HELP: &[(Kind, &str)]` table and a 20-line formatter. | C (cold path; still needs the evidence) |
| **Versions and pins scattered** | A tool version is declared once and referenced. | `npm install --global @pkl-community/pkl@0.27.2` is spelled in 8 Pkl files (`tools/config/pkl/parts/check/coverage.pkl:24`, `…/pr-rust-products.pkl:102`, …), while the catalog pins `0.30.2` (`pnpm-workspace.yaml:164`). → `ci.pklCli` in `Actions.pkl`, with the reason the fixture runtime stays on 0.27.2. | N2 (regenerated YAML is identical) |
| **Repo paths in code** | A path is built from one root constant, and a path that names another unit's file goes through that unit's exported constant. | 16,361 repo-path literals; the worst file is `tests/tooling/tooling-test-selection.test.ts` with 166. There are **1,548 separate `const root`/`repoRoot` definitions** in `tools` and `tests`. → `import { repoRoot, repoPath } from "tools/support/repo-root.ts"`. | N2 |
| **Duplicated helpers** | One helper per concept in `tools/support`. | 69 local `sha256`/`digest` functions and 57 separate `git ls-files` spawns, while `tools/support/git-tracked-files.ts` exists. | N2 |
| **Hashes in code** | Digests live in data files under `tests/_fixtures/**` with a schema, never as string literals in `.ts` or `.rs`. | `tests/differential/typechecker-registry.ts` (101 hashes) and `crates/vize_canon/tests/fix_history_diagnostics.rs` (97). | test-only |
| **Hand-rolled argv parsing** | TS tools use `node:util` `parseArgs` with a declared option table. | `tools/commands/ci/source-file-lengths.ts:284-300` (an if/else chain); 564 files read `process.argv` and 82 use `parseArgs`. | tools-only |

**Not hardcoding**: generated catalog data with a source header (`crates/vize_maestro/src/ide/style/data/*.rs`, generated by `generate-css-catalog.ts`), or reference vectors with provenance (`npm/ui/src/families/charts/chart-shape/shape-d3-vectors.ts`). Keep them, but they should live in a `data/` or `vectors/` file named as such, never mixed into logic.

### 1.5 Vertical whitespace and line breaks

The formatter owns horizontal layout. These rules own vertical layout, which no formatter in the repo enforces today.

1. **Between items**: exactly one blank line between top-level items, impl members and class members.
   Exceptions: a run of `use`/`import` lines, a run of `mod` declarations, a run of one-line `const`s that form one table, and consecutive one-line `type` aliases.
   Example `davinci/vize_l4/src/targets/dom/artifact/buf.rs:193-215`: about 20 three-line `use_*` methods are packed with no blank lines.
   → one blank line between them, or better, delete the wrappers and call `buf.mark(Helper::WithKeys)` (class C).
   Today 1,808 Rust items follow a `}` with no blank line.
2. **Inside a function**: one blank line separates steps, meaning the paragraphs a reader would name. Put one before the final `return` or tail expression when the body is more than about 8 lines, and one after a guard block (`let … else { return }` or an early-return `if`) that precedes the main work. No blank line directly after `{` or before `}`.
   Example: `crates/vize_atelier_sfc/src/script/define_props_destructure/collector.rs:345` `collect_from_expression` is 373 lines with no blank line. Today 1,686 functions are over 80 lines with no blank line.
3. **Struct fields and enum variants**: no blank lines between single-line items. Put one blank line between fields when **any** field in the struct has a doc comment of two or more lines.
   Example `buf.rs:15-40`: `preferred_visits` carries a 6-line doc that runs straight into `/// Whether…`. → blank lines between the documented fields.
4. **`match` arms**: one-line arms are grouped without blank lines. Arms with a multi-line body are separated by one blank line.
5. **Imports**, in three groups separated by one blank line and in this order:
   - Rust: `std`/`core`/`alloc`, then external crates (including other `vize_*` crates), then `crate`/`super`/`self` and local modules.
   - TS: `node:` built-ins, then packages, then relative paths.

   Type-only imports are separate `import type …` statements, which is already the majority style: 3,942 statements against 1,464 inline `type` specifiers.
   Today only 2,900 Rust files are grouped by kind, while 4,089 mix them. Example `crates/vize/tests/check_cli.rs:5-21`: `use` and `mod` interleave, `use {vize_carton::…, vize_l0::cstr};` uses a brace root, and `std` comes after `crate`. →
   ```rust
   mod empty_inputs;
   mod global_component_value_imports;
   …

   use std::io::{BufRead, Write};
   use std::path::Path;
   use std::process::Command;

   use vize_carton::corsa_resolver::platform_suffix;
   use vize_carton::path::canonicalize_non_verbatim;
   use vize_l0::cstr;

   use crate::support::{corsa_path, corsa_requirement, vite_config};
   use stubs::{write_test_vite_stub, write_test_vue_runtime_dom_stub, write_test_vue_stub};
   ```
   Also `crates/vize_fresco/src/text/wrap.rs:3-5` (`super::` before an external crate, all in one group).
6. **Order inside a file**: module docs, `mod` declarations, `use` groups, public types, the public entry function, then private helpers in call order, then `#[cfg(test)] mod tests;` last.
7. **Line breaks the formatter leaves to you**:
   - Never put two statements on one line.
   - In TS, `if (x) return y;` on one line is allowed only for guard returns (oxfmt keeps it). Otherwise use braces on their own lines.
   - Break a boolean chain of three or more operands into one operand per line, operator first (rustfmt does this past the width; do it earlier when the operands are long).
   - Do not exceed 100 columns in a string literal: split long user-facing text with `concat!` (Rust) or a template literal at word boundaries. Formatters will not do this, and 29,100 lines are over 100 columns today, mostly data and prose strings.

### 1.6 Tests

- **Rust unit tests**: inline `#[cfg(test)] mod tests { … }` at the end while the file stays ≤ 500 lines. Otherwise use `<module>/tests.rs`, or `<module>/tests/<topic>.rs` for several topics. That is the CONTRIBUTING rule. **Retire the other shapes**: 639 sibling `*_tests.rs` files (`crates/vize_maestro/src/server/state/project_contexts/freshness_tests.rs`) and 47 `tests_*.rs` files (`crates/vize_croquis_cf/src/analyzer/tests_basic.rs`). Moving them is test-only, so the `vize` binary hash is identical.
- **Rust integration tests** (`crates/*/tests/*.rs`): one binary per product surface, with modules inside (`tests/check_cli/…`), not one file per issue. Every file in `tests/` is a separate test binary, which costs link time.
- **TS tests**: package tests sit next to their source as `foo.test.ts`. Repository tooling tests live in `tests/tooling/<area>/`, with the same area names as the code they test (`lsp/`, `native/`, `typecheck/`, `lint/`, `docs/`, `release/`, …). Today `tests/tooling` holds 1,832 files in one flat directory.
- **Names**: say the behaviour, not the ticket. Example `crates/vize_canon`'s `typed_options_plain_and_mixed_props_reject_wrong_and_accept_repair_while_setup_js_stays_denied` (94 characters) → module `typed_options`, test `plain_and_mixed_props_accept_only_the_repair`.
- **Assertions**: exact oracles (the assertion lint already enforces this). One behaviour per test. Shared setup goes in builders such as `create_cli_project(...)`, not in 200-line test bodies (`tests/_helpers/apps.ts` has a 204-line function).
- **Fixtures**: one directory per case, with a `case.json`/`manifest` row that holds the issue number, source and expected output. Fixture bytes are oracles: never reformat them.

---

## 2. Rust

On top of §1:

- **Modules**: `foo.rs` + `foo/` (2018 style). There are 78 `mod.rs` files; move them (class N1, but see the path-string note in `plan.md` §3.3). No `#[path]` (already enforced).
- **Visibility**: the narrowest that compiles. `pub(crate)` inside a crate. `pub` only for items re-exported from `lib.rs`. Each `lib.rs` starts with a "start here" doc that names the entry function and root type (level-IR naming decision, 2026-11-24).
- **Function shape**: guards first (`let … else`), then the main path. Nesting deeper than 4 levels means extract a function (10,135 Rust code lines sit at 7+ indent levels today; `deep_indent` in the inventory).
- **Parameters**: 143 `#[expect(clippy::too_many_arguments)]` exist. New code passes a context or options struct instead (class C for existing code; do it while rewriting).
- **Suppressions**: `#[expect(lint, reason = "…")]` only (already enforced). The reason is a sentence about this code, not `reason = "fixtures use std strings"` copied into 376 files: put a shared expectation once on the `mod` that hosts them.
- **Strings**: `vize_l0::String`/`cstr!`, as today. Keep long text in `const` tables (§1.4).

**rustfmt, enforceable now (stable).** Add `rustfmt.toml` at the root (there is none today):

```toml
style_edition = "2024"
max_width = 100
newline_style = "Unix"
use_field_init_shorthand = true
use_try_shorthand = true
```

These are no-ops for code that already passes clippy, so they cost no churn. They write the contract down and stop drift from the floating `toolchain = "stable"` in `tools/config/pkl/parts/check/fmt-rust.pkl:10`. Also change that job to the pinned toolchain, so local `cargo fmt` and CI agree.

**rustfmt, nightly-only (maintainer decision Q3).** These options are unstable on rustfmt for the pinned 1.100 toolchain:

```toml
# rustfmt.nightly.toml, used by `fmt-rust` with a pinned nightly rustfmt only:
#   cargo +nightly-YYYY-MM-DD fmt --all -- --config-path rustfmt.nightly.toml --check
unstable_features = true
group_imports = "StdExternalCrate"   # §1.5 rule 5
imports_granularity = "Module"       # one `use` per module path; no `use {a::x, b::y};`
format_code_in_doc_comments = true
normalize_doc_attributes = true
hex_literal_case = "Upper"
format_macro_matchers = true
```

Once a file is in this form, stable `cargo fmt` keeps it: stable rustfmt sorts within existing blank-line groups and never merges them. So contributors without nightly still produce passing code. Do **not** enable `wrap_comments` or `normalize_comments`: they re-flow every comment, conflict with byte custody, and add no meaning. A pinned nightly rustfmt can come from the Nix shell (rust-overlay `rust-bin.nightly.<date>.minimal` with the `rustfmt` extension) for the `fmt-rust` job only. The compiler and clippy stay on 1.100.

**clippy, enforceable now**, in `[workspace.lints.clippy]` / `.cargo/clippy.toml`. Each lint lands only after a measured run shows zero hits or after a sweep removes them. **None may be landed with mass `#[expect]`s**: that grows over-limit files.

| Setting | Why | Precondition |
| --- | --- | --- |
| `disallowed-names = ["..", "tmp", "temp", "data2", "stuff", "thing", "foo2"]` (clippy.toml) | generic locals | measure, then sweep test code |
| `mod_module_files = "deny"` | ends `mod.rs` | after the 78 moves |
| `items_after_statements = "deny"` | items declared mid-function hurt reading order | measure |
| `unreadable_literal = "deny"` | `1_000_000`, not `1000000` (N0: same tokens, different spelling) | measure, sweep |
| `semicolon_if_nothing_returned`, `redundant_else`, `needless_continue`, `manual_let_else` | shape | class C sweeps; new code only at first |
| keep `too-many-arguments-threshold = 7` | lowering it would need expects | none |

`too_many_lines`/`cognitive_complexity` are **not** turned on in clippy: existing violators would need `#[expect]` lines in over-limit files. The custom `function-lengths` ratchet does that job (§8).

---

## 3. TypeScript and JavaScript

- **Language**: new tooling is `.ts`, never `.mjs`/`.cjs`/`.js` (already enforced for new files). There are 657 retired `.mjs` files to port, 225 of them `tests/tooling/*.test.mjs`.
- **Module shape**: imports, then module constants, then exported entry functions, then private helpers in call order. A script's top-level side effects live in one `main()` called at the bottom (as in `source-file-lengths.ts:278-337`), never scattered through the file.
- **Functions**: `function` declarations for named module-level functions, and arrows for callbacks. Prefer an options object once there are more than 3 parameters.
- **Types**: `interface` for object shapes, `type` for unions and aliases. No `any`: use `unknown` and narrow.
- **Errors**: tools `throw` and fail fast through `main()`. No `process.exit` deep inside helpers.
- **Comments**: JSDoc `/** */` on exported API in published packages (`npm/*`), where it carries information (§1.3). Plain `//` elsewhere.

**oxfmt (Vite+ `fmt`)**: keep `quoteProps: "preserve"` and the default width. Turn on import sorting so §1.5 rule 5 is mechanical. The option names below are from the oxfmt `experimentalSortImports` schema; **verify them against oxfmt 0.63** before landing:

```ts
fmt: {
  quoteProps: "preserve",
  experimentalSortImports: { newlinesBetween: true, order: "asc", ignoreCase: false },
  experimentalSortPackageJson: false, // package.json bytes are pinned by custody tests
  …
}
```

**oxlint 1.78 (Vite+ `lint`).** Rules to add; verify each name in `oxlint --rules` for 1.78. Scope them with `overrides` to `npm/*/src/**`, `tools/**`, `tests/**`, `playground/src/**` and `docs/{theme,scripts}/**`; fixtures stay ignored as they are today.

```ts
rules: {
  "unicorn/filename-case": ["error", { case: "kebabCase" }],          // §1.2 (after the rename sweep)
  "typescript/consistent-type-imports": ["error", { fixStyle: "separate-type-imports" }],
  "import/no-duplicates": "error",
  "eslint/max-depth": ["error", 4],
  "eslint/max-params": ["error", 4],
  "eslint/max-nested-callbacks": ["error", 3],
  "eslint/no-nested-ternary": "error",
  "eslint/no-magic-numbers": ["error", { ignore: [-1, 0, 1, 2], ignoreArrayIndexes: true,
                                         ignoreDefaultValues: true, enforceConst: true }],  // npm/*/src only
  "eslint/max-lines-per-function": ["error", { max: 120, skipBlankLines: false, skipComments: true }],
  "eslint/no-warning-comments": ["error", { terms: ["fixme", "xxx", "hack"] }],
}
```

`max-lines-per-function` and `no-magic-numbers` start as a ratchet (§8), not as errors: oxlint has no baseline file, so the ratchet tool runs oxlint with `--format json` and compares counts per file against `tools/config/readability-baseline.json`.

**Custom rules as an oxlint JS plugin.** `@oxlint/plugins` 1.78 is already in the catalog and used by `npm/linter/oxlint`. A small repo-local plugin, `tools/support/lint/house-style-plugin.ts`, adds rules no formatter has:

- `house/no-issue-archaeology`: `#\d{3,}` or `issue \d+` in a comment, unless the comment is `TODO(#n)`.
- `house/no-history-narration`: the phrase list in `scripts/inventory.py` `HISTORY_RE`, as a warning that turns into an error per area after its sweep.
- `house/no-banner-comments`.
- `house/padding-between-members`: §1.5 rules 1 and 3 for classes, interfaces and object-literal method groups.
- `house/blank-line-after-guard`.

---

## 4. Vue SFCs

- Block order `<script setup lang="ts">`, `<template>`, `<style>`. 979 of the 991 authored SFCs that have both a script and a template already follow it. The 12 template-first files (`npm/ui/src/families/typography/text/text.vue`, the `breadcrumb/*.vue` set and others) flip. Block order changes source offsets in the compiled output and its source maps, so treat the flip as class C, with tests and a built-package diff, not as N0.
- `defineProps<{ … }>()` type form only (maintainer rule, 2026-11-18). This is already true: 0 runtime `defineProps` in authored SFCs. Fixtures whose subject is the runtime form keep it.
- Inside `<script setup>`: imports, then props/emits/models, then composables, then local state, computed, functions and lifecycle hooks, each group separated by one blank line.
- Template: one attribute per line once an element has more than 3 attributes or passes 100 columns (vize's own formatter decides; dogfood `vize fmt`).
- The opinionated lint (`opinionated-source-lint.config.ts`) keeps its repository-local `off` list. The readability program removes entries from that list one by one as apps migrate; each removal is a sweep PR.

## 5. Pkl

- **Format**: `pkl format`. Pkl 0.30 has a formatter; verify `pkl format --help` with the catalog's `@pkl-community/pkl` 0.30.2. Run it on `tools/config/pkl/**`, then regenerate, and the generated YAML and shell must be byte-identical (class N0, proven by `repository-config.ts --check`).
- **Names**: one Pkl module per workflow or part, named for the subject. Shell-step modules are named by what they do in ≤ 6 words. `install-the-exact-release-artifacts-and-initialize-a-fresh-project-2.ts` → `install-release-and-init-project.ts`. There are no `-2` suffixes: two steps that share a name are one step with a parameter. The workflow lane (K) owns this.
- **Constants**: versions, runner labels, action pins and timeouts live in `Actions.pkl` (§1.4 Pkl example). A literal version in a part file is a defect.
- **Generated files** carry a "do not edit" header that names their source (user decision 11-24). Readability edits change the Pkl source and regenerate; nobody edits `.github/**` by hand.

## 6. Shell, Nix and editor scripts

- **Shell** (about 150 hand-written files; 1,139 more are generated from Pkl): `#!/usr/bin/env bash` and `set -euo pipefail` (`ShellScript.pkl`'s `strictMode`). Quote every expansion. Functions are `snake_case`. Anything longer than about 30 lines or with real logic becomes a `.ts` program (the workflow plan's K lane). Add `shellcheck` to the Nix shell and run it on hand-written scripts. Generated scripts are checked through their Pkl renderer.
- **Nix**: `nixfmt` (already the flake formatter, `flake.nix:30`). Add a `nix fmt -- --check` step.
- **Lua/Vim/Elisp/Scheme** (`editors/nvim`, `editors/vim`, `editors/emacs`, `tools/scheme`): about 3,000 lines in all. Use the language's standard style (`stylua` defaults for Lua). The comment rules in §1.3 apply. `tools/scheme` is slated for deletion in the tools plan.

## 7. Code in docs

- Code blocks in `docs/content/**` and `docs/davinci/**` follow the language rules above and must compile or parse. Add a docs check that extracts fenced `rust`/`ts`/`vue` blocks and parses them with the vize parsers.
- Dated records (`docs/davinci/decisions/**`, `plan/phase-*-records/**`, ledgers) are history. **Never rewrite them for style**; they keep old names on purpose.
- Generated rule pages embed **source line anchors** (`…/no_v_text_v_html_on_component.rs#L34`, 1,864 anchors in `docs/content/**`). Any comment edit above a rule's `META` shifts them. Regenerate with `docs/scripts/generate-patina-rules-page.ts` in the same PR, or, better (open question Q9), link to the file without `#L`.

---

## 8. Enforcement matrix

| Rule | Tool | Setting or mechanism | When |
| --- | --- | --- | --- |
| File ≤ 500, no growth | `source-file-lengths.ts` | exists; drop the `tools/commands`/`tools/support` exemption | after the tools move |
| Function ≤ 120, no growth | **new** `tools/commands/ci/readability-ratchet.ts` | per-file counts in `tools/config/readability-baseline.json`, shrink-only (same pattern as `assertion-ratchet.tsv`, `skeleton-todos.toml`) | toolkit PR |
| Issue numbers, history phrases, banners, restating comments | the same ratchet; for Rust it reads comments with the inventory tokenizer; for TS, the oxlint JS plugin | shrink-only per file; a file at 0 must stay at 0 | toolkit PR |
| Issue numbers in file names | ratchet: a shrink-only list of today's 442 | toolkit PR |
| No `*_tests.rs`/`tests_*.rs`/`mod.rs` | ratchet, then clippy `mod_module_files` | toolkit PR; deny after the sweep |
| Import groups (Rust) | `rustfmt.nightly.toml` (if approved), else the ratchet using `scripts/rust-import-groups.py` logic | after the format sweep |
| Import groups and `import type` (TS) | oxfmt `experimentalSortImports` + `typescript/consistent-type-imports` | per area, with its sweep |
| Blank line between items | Rust: ratchet (`items_no_blank` counter); TS: `house/padding-between-members` | per area |
| Magic numbers | TS: `no-magic-numbers` (ratchet); Rust: ratchet on numeric literals ≥ 3 digits outside `const`/`static`/tests | per area |
| Kebab-case TS files | `unicorn/filename-case` | after the rename sweep |
| `.mjs` | `retired-tooling-languages.ts` (exists), shrink-only | exists |
| Codenames | `levels.codenames` gate (exists) | exists |
| Generated-file header | `repository-config.ts --check` asserts the header | workflow lane |
| Byte custody | **new** `tools/commands/ci/byte-custody.ts`: every source whose bytes a test pins must be listed in `tools/config/byte-custody.json`, with its owning test; a PR touching a listed file must touch the owner too | toolkit PR |

## 9. What this guide never touches

- `upstream/forks/**`, vendored code, `node_modules`, `target`.
- Fixtures and snapshots (`fixtures/`, `_fixtures/`, `__fixtures__/`, `snapshots/`, `expected/`, `test-fixtures/`, `tests/fuzz/regressions/`). They are oracles.
- Generated files: the 38 in-scope files with a generated header (list: `data/generated-files.txt`), `.github/workflows/*.yml`, `.github/scripts/**` (generated from `tools/config/pkl/scripts/**`), `npm/cli/src/types/generated.ts`, `npm/native/index.{js,d.ts}`, the rule pages. Edit the source and regenerate.
- Byte-custody sources until the custody procedure releases them (`plan.md` §4).
- Translations (`docs/content/{ja,zh-CN,pt-BR,fr}/**`), except through their generators.
