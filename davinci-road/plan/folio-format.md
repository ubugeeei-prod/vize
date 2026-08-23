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

## The repro page (`[repro]`, P2-13)

The crash reproducer the ICE policy writes
(`crates/vize_davinci/src/folio/repro.rs`), hand-written because two of its
decisions are semantic: `failed-pass=` may print an **empty value** (a panic
caught outside a driven pipeline is not attributable to a pass), and the
`[repro.artifact]` section is **verbatim and terminal** — everything after
its header line, byte for byte to end of input, is the embedded last-good
stage dump, so the section must come last and is exempt from rule 4's
one-blank-line law. That exemption is the only way to embed an arbitrary
dump (blank lines and `[`-prefixed lines included) without an escaping
scheme. Header scalars are `pipeline=` (validated against the P2-2 pipeline
grammar at parse time), `failed-stage=`, `failed-pass=`, `reason=`
(newline-normalized by the writer; scalar values stay line-atomic) and
`artifact-stage=` (`source` = authored input verbatim); `[repro.config]` is
a sorted `key=value` map. A missing final newline on the artifact is added
by the first print, and `ReproFolio::normalize` applies the same to
hand-built values — the `CroquisFolio::normalize` precedent. Round-trip
laws pinned by `crates/vize_davinci/tests/repro_folio.rs`.

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

## Disegno page (P2-5a; expression payloads P2-5b)

The S2 stage dump: an owned document model (`DisegnoFolio`,
`crates/vize_disegno/src/folio.rs`) of one op tree. Hand-written under the
"Derived pages" boundary, because the derived grammar is flat (header
scalars plus one-level sections) while the S2 artifact is region-nested by
its central design decision — ops own their regions — and flattening the
tree into derivable lines would move structure validation outside `parse`,
stripping its 1-based line numbers.

Two sections, fixed order: a `[disegno]` header whose single `ops=` field
is the printer's **computed** statement of the total op count (region ops
plus attached bindings, all levels; parse validates the integer and
discards it — normalization by the first print), then `[disegno.ops]`
holding the tree, omitted when empty. Nesting is two-space indentation;
a shallower line closes every deeper op. Under an element or component the
grouping is fixed: `attr` lines, then attached bindings (`ui.model`,
`vue.directive`, `vue.css-bind`, `vue.sync`, `vue.slot-scope`), then children. Under `ui.if` only `branch` lines are
legal; under `ui.model` only `attr` lines. Blank lines are separators and
vanish; every other spelling is strict with exact, tested rejections.

One line per op (`[]` optional; `<expr>` is an expression payload token,
below):

| line                                                                                    | notes                                            |
| --------------------------------------------------------------------------------------- | ------------------------------------------------ |
| `ui.element <tag>[ ns=<svg\|mathml>] @s:e`                                              | HTML namespace elided                            |
| `ui.component <name> @s:e`                                                              | same body grouping as an element                 |
| `ui.text <quoted> @s:e`                                                                 |                                                  |
| `ui.interpolation <expr> @s:e`                                                          |                                                  |
| `ui.if @s:e`                                                                            | `branch [<expr> ]@s:e` lines beneath             |
| `ui.for source=<expr> value=<expr>[ key=<expr>][ index=<expr>] @s:e`                    | region beneath                                   |
| `ui.slot name=<quoted>\|name=<expr> @s:e`                                               | fallback region beneath                          |
| `ui.model read=<expr> write=<expr> @s:e`                                                | `attr` lines beneath                             |
| `vue.directive <quoted>[ arg=<quoted>\|arg=<expr>][ mods=<quoted>][ value=<expr>] @s:e` | modifiers comma-joined inside quotes             |
| `vue.css-bind value=<expr> @s:e`                                                        | SFC style `v-bind()`; span is CSS-block-relative |
| `vue.sync name=<quoted>[ mods=<quoted>] value=<expr> @s:e`                              | Vue 2 `:foo.sync`; name is static                 |
| `vue.slot-scope[ name=<quoted>][ params=<expr>] @s:e`                                   | Vue 2 `slot-scope` / `scope` sugar               |
| `attr <name>[=<quoted>] @s:e`                                                           | bare name for boolean attributes                 |

**Expression payloads** (P2-5b): every expression position serializes as
owned text + span, never an AST, because arena references cannot persist
across a compile (P1-11) — a `js` payload re-parses into the arena on
load (`vize_disegno::expr::JsExpr::parse_in`; the total fallback is
`ExprRef::parse_js_in`, which loads unadmitted text as `opaque` with the
text-classified reason).

| token                              | payload                                                                                                                                                |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `js(<quoted> @s:e)`                | retained-AST text (the P1-5 admission: one complete TS expression covering the text) plus its authored span                                            |
| `opaque(<reason> <quoted> @s:e)`   | the escape variant: classified reason + exact text + span; reasons are `for-value`, `multi-statement`, `nesting-refused`, `parse-rejected`, `compound` |
| `foreign(<dialect> <quoted> @s:e)` | dialect id + text + span (type-only until phase 6; side tables have no spelling until the phase-6 dialect contract defines one)                        |
| `vue.filter(<quoted> @s:e)`        | Vue 2 pipe-filter chain as authored text (`msg \| capitalize`); the pipe is not bitwise-OR. Reloads via `VueFilterExpr::parse_in`                      |

Quoted strings escape `\\`, `\"`, `\n`, `\r`, `\t`. `Display` elides every
` @s:e` span — line tails and the spans inside expression payloads alike —
and nothing else. Documented edges (same rule as derived pages):
attribute names containing `=`, a space or `"`, modifier names containing
`,` or `"`, dialect ids containing a space, `)` or `"`, and values
embedding other control characters are outside the contract. The folio
models the dump, not the analysis: tree shape is validated, semantic
invariants (branch ordering, region well-formedness beyond the grammar)
belong to the S2 verifier (P2-6). The committed reference page is
`crates/vize_disegno/tests/fixtures/reference.folio`, pinned by TS-16 in
`crates/vize_disegno/tests/folio_laws.rs` (which also pins every opaque
reason spelling both directions) and mirrored from a live arena tree in
`tests/folio_mirror.rs`; the arena-reset replay law is
`tests/expr_replay.rs`.

## S2 verifier invariants (P2-6)

The semantic invariants the disegno grammar deliberately does not encode,
checked by `vize_disegno::verify` between passes in debug/CI builds only
(guardrail 5: verification never ships — the release shape of
`VerifyObserver` is a ZST with empty check bodies, const-asserted at the
type). Checks are local in the GHC `-dcore-lint` sense — a line plus the
facts it and its owner already declare, no global inference — and run in
one page-order walk, so an aggregated report is deterministic. A violation
renders as one line, `{code} @{start}:{end} {message}`, canonical `en`
locale; a between-pass failure panics with the report headed by the
offending pass (`` S2 verifier: {n} violation(s) after `{stage}.{pass}` ``).

| code   | rigor      | invariant                                               |
| ------ | ---------- | ------------------------------------------------------- |
| S2V001 | structural | a span never runs backwards (`start <= end`)            |
| S2V002 | structural | a nested line's span stays inside its immediate owner's |
| S2V003 | structural | every `NodeId` a side table references resolves         |
| S2V004 | canonical  | `ui.if` owns at least one branch                        |
| S2V005 | canonical  | the leading branch of `ui.if` carries a condition       |
| S2V006 | canonical  | an unconditional branch is the trailing branch          |

**Rigor follows `PassKind`.** The structural set holds after every pass;
the canonical set additionally holds from the first `MandatoryLowering`
pass on — the kind that canonicalizes
(`crates/vize_davinci/src/pass/kind.rs`) — and `MandatoryDiagnostic` /
`Optional` passes never change the rigor. The set grows with the passes
that establish more canonical form (P2-9); a new invariant lands here
first, with its code.

**Node numbering (S2V003).** S2 ids are dense and page-ordered: every op
line top to bottom (`attr` and `branch` lines carry no id), so a `NodeId`
resolves iff its index is below the artifact's total op count — the same
count the printed `ops=` header states. P2-8's lowering mints ids in this
order. A dangling reference renders with the `%index` display form and
the artifact-level span `@0:0` (the reference has no source anchor of its
own).

**Expr-ref liveness** is the one check with no code of its own: it reuses
the P1-11 debug arena-generation stamp (`Allocator::stamp` /
`assert_stamp_current`, `crates/vize_carton/src/allocator/generation.rs`)
and fails with that mechanism's own panic. One stamp covers the whole
artifact today because `ExprSlot` is zero-sized; the P2-5b seam is
`VerifyObserver::check_live`, where the walk validates each expression
position's stamp once `ExprRef` gives the positions identity.

**Invalid fixtures (TS-18).** `crates/vize_disegno/tests/fixtures/invalid/`
holds hand-built pages that are grammar-valid and semantically invalid,
each committed beside its exact expected rendering (`.expected`,
whole-file equality, no partial matching). The harness is
`crates/vize_disegno/tests/verifier_fixtures.rs`; the id-resolution and
liveness lanes, which no page text can encode, are pinned with the same
exact oracles in `tests/verifier_observer.rs`.
