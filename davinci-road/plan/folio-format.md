# Folio format — normalization contract

> [!NOTE]
> The "test-mode printer" contract for Davinci folio dumps (P0-10): which
> text is canonical, what normalization the printer applies, and what
> `Display` mode elides. The trait lives in
> `crates/vize_davinci/src/folio.rs`; the hand-written page is the croquis
> folio (`crates/vize_davinci/src/folio/croquis.rs`), which absorbs the
> croquis "VIR" dump (`crates/vize_croquis/src/croquis/vir.rs`), and since
> P2-4 `#[derive(Folio)]` generates pages under the "Derived pages"
> contract below.

## The contract

`trait Folio` has two operations and two modes:

- `print(&self, w, mode) -> fmt::Result` — render the value.
- `parse(input: &str) -> Result<Self, FolioError>` — read `Full`-mode text
  back; errors carry a 1-based line number.

**Equality laws are mode-explicit:**

| mode      | purpose              | laws                                                                               |
| --------- | -------------------- | ---------------------------------------------------------------------------------- |
| `Full`    | injective, parseable | `print(parse(t)) == t` for canonical text `t`; `parse(print(v)) == v` structurally |
| `Display` | human-oriented       | spans and default markers elided; **no** round-trip law                            |

Canonical text is what `print(Full)` emits. Non-canonical input (unsorted
lists, reordered sections, raw renderer output) is **normalized by the
first print, by design**: `parse` is lenient exactly where the printer
normalizes, and `parse` itself returns normalized values, so the
structural law quantifies over normalized values (`CroquisFolio::normalize`
canonicalizes hand-built ones). `davinci-opt --roundtrip <file>` checks
the byte-identity law; exit 0 means the file is canonical.

## Normalization rules (both modes)

1. **Sorted map iteration** — every name list the renderer produces by
   iterating a hash map is emitted sorted (lexicographic, byte order):
   `[bindings]` names within each group, `[extern]` binding lists, and
   `[scopes]` per-scope binding lists. Order-bearing lists are _not_
   sorted: section entry order, binding-group order, and `[scopes]` parent
   references (the first parent is the lexical parent).
2. **Stable sequential ids** — `[scopes]` display ids are renumbered
   sequentially per prefix (`~` universal, `!` client-only, `#`
   server-only) in entry order, and parent references are remapped.
   Renderer output already satisfies this; the rule makes it a printer
   guarantee instead of a renderer accident.
3. **Fixed section order** — `[vir]`, `[surface.props]`,
   `[surface.emits]`, `[surface.models]`, `[surface.expose]`,
   `[surface.slots]`, `[macros]`, `[reactivity]`, `[extern]`, `[types]`,
   `[bindings]`, `[scopes]`, `[errors]`. Parse accepts any order (each
   section at most once); print always emits this order.
4. **Empty sections are omitted**, every printed section (the header
   included) is followed by exactly one blank line, and lines are
   LF-terminated. Extra blank lines between entries of non-verbatim
   sections are separators and vanish on the first print.
5. **Counts are carried, not checked** — the `[vir]` header's `scopes=` /
   `bindings=` counts are the renderer's statement about the analysis; the
   folio reprints them verbatim and enforces no cross-section invariant
   beyond scope-id uniqueness and parent-reference resolution.

## `Display` elision

`Display` drops every `@start:end` span (`[macros]`, `[types]`,
`[scopes]`, `[errors]`, and span-only `[surface.expose]` /
`[surface.slots]` fallback lines) and the trailing `=` default marker in
`[surface.props]`. Sections left empty by elision are omitted. Everything
else prints as in `Full`.

## Derived pages (`#[derive(Folio)]`, P2-4)

The derive generates the **mechanical trio only** — print, parse, field
order — for an owned document struct; anything semantic stays hand-written,
which is why `CroquisFolio` keeps its hand impl. One generated grammar for
every derived type `T`:

- `[page]` header section, where `page` is the kebab-case type name
  (`BudgetObserver` → `[budget-observer]`). Required, and first.
- Scalar fields print as `name=value` lines inside the header, in field
  declaration order. All are required; parse accepts them in any order,
  each at most once. Values go through `FolioValue`
  (`crates/vize_davinci/src/folio/value.rs`): `bool`, the integers, and
  `vize_carton::String` — an unsupported field type is a compile error.
- A `Vec<T>` field prints as a `[page.field]` section, one entry per line,
  **order preserved** (order-bearing lists are never sorted, rule 1).
- An `FxHashMap<K, V>` field prints as a `[page.field]` section of
  `key=value` lines **sorted by printed key** (lexicographic, byte order —
  rule 1). Duplicate keys are a parse error.
- Rules 3–5 apply as written: print always emits declaration order, parse
  accepts sections in any order (each at most once), empty sections are
  omitted, every printed section ends with exactly one blank line, LF only,
  and parse errors carry 1-based line numbers.
- Rule 2 (stable sequential ids) is **not applicable**: renumbering
  requires knowing which field is an id, which is a semantic decision the
  derive refuses to make. A derived type carrying id references needs a
  hand-written page.

`Display` on a derived page prints the same canonical text as `Full`:
elision is semantic, so the derive elides nothing — and `Display` carries
no round-trip law regardless.

**Documented edges (outside the contract, strict where possible):** values
are line-atomic, so a value embedding `\n` does not round-trip; a map key
containing `=` splits at the first one; a list entry shaped exactly like
`[section]` is parsed as a section header and **rejected** (`unknown
section`) rather than silently misparsed; an empty-string list entry prints
as a blank line and vanishes on reparse.

The first derived page is `[budget-observer]` (P2-3's counter set), pinned
by TS-16 in `crates/vize_davinci/tests/folio_derive_laws.rs` and reachable
from the CLI as `davinci-opt --stage budget-observer`.

## Croquis folio grammar

One entry per line unless noted; `[]` marks optional parts.

| section            | entry grammar                                       | notes                                       |
| ------------------ | --------------------------------------------------- | ------------------------------------------- |
| `[vir]`            | `script_setup=<bool>`, `scopes=<n>`, `bindings=<n>` | required header, must be the first section  |
| `[surface.props]`  | `{name}{!\|?}[:{type}][=]`                          | `!` required, `?` optional, `=` has-default |
| `[surface.emits]`  | verbatim line                                       |                                             |
| `[surface.models]` | verbatim line                                       |                                             |
| `[surface.expose]` | args text, or `@{start}:{end}`                      | span-only fallback detected per line        |
| `[surface.slots]`  | args text, or `@{start}:{end}`                      | span-only fallback detected per line        |
| `[macros]`         | `@{name}[<{type_args}>] @{start}:{end}`             | may span physical lines (see below)         |
| `[reactivity]`     | verbatim line (`{name}={kind}`)                     |                                             |
| `[extern]`         | `{source}[^][ {{a,b}}]`                             | `^` type-only; binding list sorted          |
| `[types]`          | `{name}[^]{t\|i}@{start}:{end}`                     | `^` hoisted; `t` type / `i` interface       |
| `[bindings]`       | `{code}:{a,b,c}`                                    | codes may repeat (`ist` is shared); sorted  |
| `[scopes]`         | `{id} {name} @{start}:{end}[ [a,b]][ < p, q]`       | ids unique; parents must resolve            |
| `[errors]`         | `{name}={kind}@{start}:{end}`                       | kind token carried verbatim                 |

**Multi-line values.** Source-derived fields can embed newlines. A
`[macros]` entry accumulates physical lines until one ends with the
` @{start}:{end}` span tail. The verbatim sections (`emits`, `models`,
`expose`, `slots`, `reactivity`) keep every interior physical line —
including blank ones — and only trailing blank lines act as the section
separator; a multi-line value there round-trips as several verbatim
entries. `[surface.props]` types are parsed per physical line (no
multi-line prop type has been observed in renderer output; the fixture
harness below is the oracle that would surface one).

**Inherited ambiguities (known, deterministic).** The `[vir]` format
predates parsing, so a few pathological values are indistinguishable in
text: a prop type ending in `=` reads as a has-default marker, an emit
name containing `:` cannot be split from its payload type (verbatim
storage sidesteps this), expose args shaped exactly like `@1:2` read as a
span fallback, and a physical line inside a multi-line macro value that
matches `[known-section]` reads as a section header. Parsing is
deterministic in every case and rejoins byte-identically where the entry
is stored verbatim; the structural law holds for every parse-produced
value.

## Fixtures

`crates/vize_davinci/tests/fixtures/croquis/` holds `.vue` inputs with
their committed canonical `.folio` dumps. The harness
(`cargo test -p vize_davinci --test croquis_folio`) re-analyzes each input
with the inspector recipe, checks the parser accepts the live renderer's
output, and asserts the committed folio equals the canonical print plus
both round-trip laws. Regenerate after a deliberate renderer change with
`UPDATE_FOLIO_FIXTURES=1`. Verify from the CLI with
`cargo run -p vize_davinci --bin davinci-opt -- --roundtrip <file> --stage croquis`.

Provenance (P0-2's fixture ladder had not landed when these were drawn, so
the inputs come from the e2e project fixtures plus two written for
coverage):

| fixture                          | source                                                                      |
| -------------------------------- | --------------------------------------------------------------------------- |
| `define-emits-type.vue`          | `tests/_fixtures/_projects/compiler-macros/src/DefineEmitsType.vue`         |
| `define-expose-type.vue`         | `tests/_fixtures/_projects/compiler-macros/src/DefineExposeType.vue`        |
| `define-model-type.vue`          | `tests/_fixtures/_projects/compiler-macros/src/DefineModelType.vue`         |
| `define-props-with-defaults.vue` | `tests/_fixtures/_projects/compiler-macros/src/DefinePropsWithDefaults.vue` |
| `define-slots-type.vue`          | `tests/_fixtures/_projects/compiler-macros/src/DefineSlotsType.vue`         |
| `directive-builtins.vue`         | `tests/_fixtures/_projects/generic-build/src/DirectiveBuiltins.vue`         |
| `existing-import.vue`            | `tests/_fixtures/_projects/typecheck-vue-imports/src/ExistingImport.vue`    |
| `invalid-exports.vue`            | written for `[errors]` / `[types]` coverage                                 |
| `normal-script-bindings.vue`     | `tests/_fixtures/_projects/generic-build/src/NormalScriptBindings.vue`      |
| `nuxt-auto-imports.vue`          | `tests/_fixtures/_projects/nuxt-template-globals/pages/index.vue`           |
| `props-destructure.vue`          | `tests/_fixtures/_projects/compiler-macros/src/PropsDestructure.vue`        |
| `props-runtime-defaults.vue`     | written for the `[surface.props]` has-default (`=`) marker                  |
| `provide-inject-symbol.vue`      | `tests/_fixtures/_projects/compiler-macros/src/ProvideInjectSymbol.vue`     |
| `top-level-await.vue`            | `tests/_fixtures/_projects/compiler-macros/src/TopLevelAwait.vue`           |
