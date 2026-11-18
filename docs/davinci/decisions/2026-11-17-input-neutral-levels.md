# Input-neutral levels: one owner per input, one module description

Tracked in [#6839](https://github.com/ubugeeei-prod/vize/issues/6839) (L3),
[#6840](https://github.com/ubugeeei-prod/vize/issues/6840) (L4) and
[#6835](https://github.com/ubugeeei-prod/vize/issues/6835)/[#6841](https://github.com/ubugeeei-prod/vize/issues/6841)
(L1 layout). Rule 7 of the Davinci charter (#13199) is the reason. It refines the
[#6855 axes](./2026-10-04-l1-neutral-axes.md), the
[JSX lowering](./2026-11-10-6885-jsx-l2-lowering.md) and the
[TSRX profile](./2026-11-09-6858-tsrx-vue-profile.md) records. Where they
disagree about placement, this record wins.

## Problem

Each input format has grown its own path through the levels, and every path is
placed differently:

- **TSRX** is a front end, just as the SFC and TSX are. Yet `vize_l4::lang::tsrx`
  (2.1k lines) contains its compile entry, component detection, the props
  shape (destructure, defaults, rest), the scope-shadowing rule, the names a
  component reads, host-code splicing and component wrapping. Its L1→L2
  lowering lives in the L2 crate (`vize_l2::lang::tsrx`), not in `vize_l1_to_l2`.
- **JSX** lowers in `vize_l1_to_l2::framework::vue::jsx`, which is the right
  place. L3 then keeps a JSX-only decision projection (`vize_l3::jsx`). L4 has a
  top-level `vize_l4::jsx` module emitter, and `vize_l4::targets::ts::module`
  splices JSX roots into host code a second time.
- **SFC** module assembly is in `vize_l4::module` (`setup.rs`, `ordinary.rs`,
  `scope.rs`, render-function names). The script compile that feeds it is in
  the legacy `vize_atelier_sfc`.
- **L1** mixes axes. The HTML-template grammar files sit at the `markup/` root
  beside the shared token and profile contracts, while `markup/tsrx/` sits one
  level deeper. `pug/` sits at the crate root and `build/` is a root sibling.
  Vue directive decomposition (`markup/directive.rs`) is a Vue hook, yet it
  lives in markup. The Vue axis is split across `dialect/{vue,vue0,vue1,vue2,vue3}`
  and `registry.rs`. Style languages are merged per syntax family
  (`css/lang/{css,nested,indented}`), not one module per language, and Vue's
  CSS `v-bind()` sits in `css/bind.rs`. `moved.rs` is a deprecated re-export
  shim. It was marked "kept for one minor release" in 0.475.0, and the crate
  is now on 0.77x.

Three different splicing mechanisms (TSRX, JSX and the TS projection), two
component-wrapping mechanisms (TSRX and SFC) and an input-specific L3 module
have produced the same kind of drift that MLIR dialect sprawl does.

## Decision

1. **Input-specific code ends at L2.** Each input has one L1 parser owner and one
   L1→L2 lowering owner. L2 holds neutral ops (`ui.*`), framework dialect ops
   (`vue.*`) and neutral facts. L2→L3, L3 and L4 never name an input format.
2. **One input-neutral module description.** Every input lowers into one
   `vize_l2::module::ModuleDef`: SFC, JSX/TSX, TSRX and any future container.
   It carries the host text and an ordered list of edits over it, plus one
   `ComponentDef` per component (defined below). L4 has a single module
   assembler that emits a `ModuleDef` for any target. Per-input modules under
   L3/L4 are removed.
3. **Component facts are L2 facts.** Detecting a component, the props it
   declares (keys, defaults, required, rest as attrs), where its setup runs
   once, which reads are prop reads (with scope shadowing applied) and which
   names a component reads are all semantics. They are computed in the input's
   L1→L2 lowering over the retained host AST, using the one L2 Program walk and
   its resolution table. L4 writes target syntax for them; it does not re-derive
   them.
4. **Front-door entries live outside the levels.** A product calls
   `vize_l1_to_l2::<owner>::lower_module(..) -> ModuleDef`, then
   `vize_l4::module::emit(&ModuleDef, target)`. An input's convenience entry
   (today's `vize_l4::lang::tsrx::compile`) moves to the product crate that
   exposes it (the CLI, the Vite/Vite+ host, the napi binding), or to the
   driver crate that replaces the `vize_atelier_*` shells under #6828. Levels
   never depend upward.
5. **L1 directories follow the #6855 axes exactly**, with one sibling per
   grammar or language and one Vue owner. Modules are named by concept, never
   by migration status (`native`). L1 does not know about products
   such as lint. The target trees are below. This
   supersedes #6855's top-level `profile/`: a host-document profile only means
   something to a markup grammar, so it stays a shared contract at the
   `markup/` root.
6. **Gates.** A new measured gate, `levels.inputs`, forbids input-specific
   modules and identifiers in L2→L3, L3 and L4, and requires the L1 layout
   below. It starts with an `owed` list that can only shrink, so no new
   misplacement can land from the day this record merges. The gate passes
   when the list is empty.

## The module description (L2)

```rust
// vize_l2::module: neutral, no input names.
pub struct ModuleDef<'a> {
    /// The authored file; every span below is file-absolute.
    pub source: &'a str,
    /// Host language of the retained script (JS/TS, with or without JSX).
    pub host: HostLang,
    /// Ordered, non-overlapping edits over `source`. Bytes between edits are
    /// copied verbatim with linked spans (one source map for the module).
    pub edits: Vec<'a, Edit<'a>>,
    pub components: Vec<'a, ComponentDef<'a>>,
}

pub enum Edit<'a> {
    /// Replace a range with the emitted render of a component (a TSRX
    /// region, a JSX root, an SFC template block).
    Render { span: Span, component: ComponentId },
    /// Replace a component definition with the target's component shape
    /// (TSRX/JSX function component, SFC `export default`).
    Define { span: Span, component: ComponentId },
    /// Remove authored bytes (erased macros, the SFC's non-script blocks).
    Erase { span: Span },
    /// Rewrite one retained read (a prop local read through the props
    /// object, a `$setup` access) from the L2 resolution table.
    Access { span: Span, access: Access<'a> },
}

pub struct ComponentDef<'a> {   // extends today's vize_l2::component::ComponentDef
    pub name: Option<&'a str>,
    pub exec: Exec,              // ReRender | RunOnce
    pub scope: RenderScope,      // Lexical | Instance
    pub span: Span,              // authored render root
    pub body: ArtifactParts<'a>, // the lowered render
    pub setup: Option<Span>,     // host statements that run once per instance
    pub props: PropsDecl<'a>,    // declared keys, defaults, required, rest → attrs
    pub contract: Contract<'a>,  // emits, models, slots, expose (framework::vue::contract)
}
```

- `PropsDecl` reuses the L2 `framework::vue::contract` types the SFC route
  already fills from `defineProps`. A TSRX destructured parameter and a JSX
  component's props fill the same table.
- `Access` is the existing L2 identifier-resolution result. "This read of `x`
  is prop `key`" is a resolution fact, so scope shadowing is decided once, in
  the walk. L4 spells the access for its target (`__props.key`, `$props.key`).
- L3 decisions stay keyed by L2 NodeIds per `ComponentDef::body` (#6839). L3
  never sees edits or host text.
- `JsxChildValue` (`ui.jsx-child`) is already neutral in meaning, and TSRX
  lowers to it too. It is renamed `ChildValue` (`ui.child-value`) in a separate
  rename-only commit, so that no neutral op carries an input name.

## Inventory and destination

### L4 (`vize_l4`, 39.8k lines)

| Today                                                                       | Lines | What it does                                                                      | Destination                                                                                                                                 |
| --------------------------------------------------------------------------- | ----: | --------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `lang/tsrx.rs`                                                              |   237 | public compile entry: parse, lower, emit, typed diagnostics                       | product entry (decision 4); diagnostics vocabulary to `vize_l2::module`                                                                     |
| `lang/tsrx/component.rs`, `props.rs`, `scopes.rs`, `names.rs`               |  1.2k | component detection, props shape, shadowing, reads                                | `vize_l1_to_l2::framework::vue::tsrx` producing `ComponentDef::{setup, props}` and `Access` edits through the L2 walk                       |
| `lang/tsrx/dom.rs`                                                          |  ~400 | host copy, region splice, `defineComponent(function …)` wrapping                  | `module/` (splice) + `targets/dom/component.rs` (wrapping), input-neutral                                                                   |
| `lang/tsrx/erase.rs`                                                        |     – | TS erasure of the whole compiled module                                           | `module/erase.rs`, shared by every input with a TS host                                                                                     |
| `jsx.rs`, `jsx/{admission,helpers,whitespace,write}.rs`                     |   559 | whole JS JSX modules from L3 JSX decisions                                        | dissolves: splice into `module/`, emission into `targets/dom`; whitespace rule already applied in L1→L2                                     |
| `targets/ts/module.rs`                                                      |   184 | JSX/TSX roots replaced in host text for type checking                             | `targets/ts` emits a `ModuleDef` through `module/` (same splice as every target)                                                            |
| `module.rs`, `module/{setup,ordinary,scope,imports,output}.rs`              |  1.4k | SFC assembly: imports, hoists, setup link, ordinary default rewrite, render names | stays as the one assembler, generalized to `ModuleDef` edits; `setup`/`ordinary` become `Define`/`Erase` edits produced by the SFC lowering |
| `targets/dom/artifact/sfc_style.rs`                                         |     – | SFC style-block carriers                                                          | style facts arrive through `ModuleDef`; the target reads neutral scope/CSS-var facts                                                        |
| `framework/vue`, `targets/{dom,ssr,vapor,vue2}`, `expr`, `runtime`, `write` |     – | targets, Vue runtime vocabulary, writer                                           | stay. `targets/vue2` stays a framework-runtime target under the #6829 rule                                                                  |

### L3, L2→L3

| Today                                              | What it does                            | Destination                                                                                                |
| -------------------------------------------------- | --------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `vize_l3::jsx` (`jsx.rs`, `jsx/build.rs`)          | JSX decisions beside the owning L2 File | removed: the neutral L3 decisions run on each `ComponentDef::body`, whatever produced it                   |
| `JsxModuleRefusal`, `JsxFile` uses in `decision/*` | JSX-specific refusal/admission paths    | neutral refusal kinds keyed by op, not input                                                               |
| `vize_l2_to_l3` JSX/SFC mentions                   | classification comments and flags       | neutral (the `OpFamily::JsxClass` bit is already scheduled to move onto `ui.bind class` by the JSX record) |

### L2 and L1→L2

| Today                                           | Destination                                                                                                    |
| ----------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `vize_l2::lang::tsrx` (lowering, 1.1k)          | `vize_l1_to_l2::framework::vue::tsrx` (lowering is L1→L2's job; the Vue profile is a framework composition)    |
| `vize_l1_to_l2::framework::vue::jsx`            | stays; also produces `ModuleDef`                                                                               |
| `vize_l1_to_l2::{vue_file,native_file,lower}`   | SFC lowering; produces `ModuleDef` (one component, script edits from `framework::vue::{setup,contract}` facts) |
| `vize_l1_to_l2::lower::pug`                     | `vize_l1_to_l2::markup::pug` (grammar lowering beside `markup::html`)                                          |
| `vize_l2::component`                            | `vize_l2::module` (adds `ModuleDef`, `Edit`, the extended `ComponentDef`)                                      |
| `vize_l2::lang::js` (File walk, JSX body facts) | stays: the JS/TS host language is an L2 language module                                                        |

### L1 (`vize_l1`)

| Today                                                                                       | Destination                                                                                                                                                                                                                                                                             |
| ------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `markup/{lex,parse,document,native,surface,render}`, root `build.rs` + `build/`, `event.rs` | `markup/htmlish/` (the HTML-tokenizer family: document and component profiles, below)                                                                                                                                                                                                   |
| `markup/{token,profile,grammar}.rs`, `markup/entity`                                        | stay at the `markup/` root: contracts and decoding shared by every grammar. `markup/profile.rs` keeps the neutral `Document` and `Component` profiles; `VueComponent` moves to `framework/vue/profile.rs`                                                                               |
| `pug.rs`, `pug/`                                                                            | `markup/pug/`                                                                                                                                                                                                                                                                           |
| `markup/tsrx/`                                                                              | stays (`markup/tsrx/`), now a sibling of `htmlish` and `pug`                                                                                                                                                                                                                            |
| `markup/directive.rs` (Vue directive decomposition)                                         | `framework/vue/directive.rs`; the neutral `DirectiveSyntax` hook trait stays in `markup/grammar.rs`                                                                                                                                                                                     |
| `dialect/{mod,vue,vue0,vue1,vue2,vue3}`                                                     | `framework/vue/{feature, version/{v0,v1,v2,v3}, quirks}` (the #6841 composition), one Vue owner                                                                                                                                                                                         |
| `registry.rs`                                                                               | stays: the only file that names concrete variants                                                                                                                                                                                                                                       |
| `container/vue`                                                                             | stays (`container/vue`, plus `container/plain` when added): a container is a file format, not a framework                                                                                                                                                                               |
| `css.rs`, `css/{parser,list,empty}.rs`                                                      | `style/css/` (CSS lexis plus the cssparser production owner)                                                                                                                                                                                                                            |
| `css/structure*`, `css/classes.rs`                                                          | `style/structure*`, `style/classes.rs`: the two language-neutral readers and the selector grammar                                                                                                                                                                                       |
| `css/lang/nested.rs` (SCSS + Less)                                                          | `style/scss.rs`, `style/less.rs` over a shared `style/nesting.rs`                                                                                                                                                                                                                       |
| `css/lang/indented.rs` (Sass + Stylus)                                                      | `style/sass.rs`, `style/stylus.rs` over `style/structure/indented.rs`                                                                                                                                                                                                                   |
| `css/lang/css.rs`                                                                           | `style/css/`                                                                                                                                                                                                                                                                            |
| `css/bind.rs` (Vue CSS `v-bind()`)                                                          | `framework/vue/style_bind.rs`                                                                                                                                                                                                                                                           |
| `embed/`, `edit.rs`, `slice.rs`                                                             | stay                                                                                                                                                                                                                                                                                    |
| `moved.rs`                                                                                  | deleted. The only in-repo user is `crates/vize_atelier_sfc/examples/n8n_compiler_custody/native.rs` (`vize_l1::parse`), which moves to `vize_l1::markup::parse` in the same commit. The breaking change is recorded in the release notes, as `vize_l1`'s experimental contract requires |

### `markup/htmlish`: one grammar, two profiles

`markup/htmlish` is the HTML-tokenizer family. It is not ".html only": the
same lexer and tree builder read an `.html` document (also in-DOM templates
and petite-vue) under the **Document** profile, and an SFC `<template>` under
the **Component** profile with Vue's attribute rule on top. The name says
"HTML-ish" on purpose. Other names were rejected: `html` (reads as `.html`
files), `template` (Pug and TSRX are template grammars too) and `vue` (an
`.html` document is not a Vue SFC).

The profiles are zero-sized markers whose associated constants are the only
place the two hosts differ (`markup/profile.rs`):

- `Document`: WHATWG HTML parsing as a browser builds the DOM.
- `Component`: the neutral component-template profile. It is HTML syntax
  without document tree construction: names keep their case, any element may
  self-close, there are no implied end tags or content models, and attribute
  whitespace is required as in HTML. Its consumers today are the published
  tokenizer facade (Armature's callback compatibility, legacy under #6828) and
  any future non-Vue component grammar (the #6829 proofs of concept).
- `VueComponent`: `Component` plus Vue's SFC attribute boundary (a closing
  quote ends an attribute, `REQUIRE_ATTRIBUTE_WHITESPACE = false`). That is a
  Vue rule, so it moves to `framework/vue/profile.rs` as `VueTemplate`. The
  neutral `markup/profile.rs` keeps `Document` and `Component`.

#### htmlish oracles and divergences

Each reading of `markup/htmlish` is a **dialect** with two references. Its
**oracle** is the reference parser whose tree we compare against. Its **norm**
is the specification that decides what counts as an error.

| Dialect    | Files                                 | Profile                        | Oracle                                                                                                | Norm                                          |
| ---------- | ------------------------------------- | ------------------------------ | ----------------------------------------------------------------------------------------------------- | --------------------------------------------- |
| `document` | `.html`, in-DOM templates, petite-vue | `Document`                     | WHATWG parsing as Chromium `DOMParser` runs it; parse5 over html5lib-tests as the browser-free engine | WHATWG HTML                                   |
| `vue`      | `.vue` `<template>`                   | `VueComponent` (`VueTemplate`) | `@vue/compiler-dom` 3.5 `parse`                                                                       | `@vue/compiler-dom` / `@vue/compiler-sfc` 3.5 |

The ledger `docs/davinci/plan/htmlish-divergences.json` is the machine-checked
record of how the dialects differ, row by row. Each row gives the behavior of
both dialects, the profile constant that switches it (if any), and one
differential fixture per dialect. A fixture that does not exist yet is `null`
and appears in the owed list as `fixture:<row>:<dialect>`.

The ledger covers the profile switches: declarations; element-name,
attribute-name, end-tag and foreign-element case; self-closing; attribute
whitespace; implied end tags including `</br>`; the `select` and table content
models. It also covers the token rules both dialects must state: raw text for
`script`/`style` and for `xmp`/`iframe`/`noembed`/`noframes`, script-data
escapes, RCDATA (`textarea`, `title`), SVG `title`, `foreignObject`,
`plaintext`, `noscript`, `template` contents, bogus comments (`<?x>`,
`</ div>`, `<!x>`), comment edges (`<!-->`, `<!--->`, `--!>`), CRLF, NULL and
character references.

A difference between the two dialects is not a vize decision: each follows its
oracle. Where **vize** departs from an oracle, the ledger lists it under
`divergences`, with a `decidedBy` link to the record that decided it, a
fixture, and a `fuzzExempt` rule that tells the oracle fuzzer which inputs not
to compare. `tests/tooling/davinci-htmlish-divergences.test.ts` checks the
ledger:

- fixture paths exist;
- every `null` fixture is in the owed list, and every `fixture:` owed entry is
  a `null` in the ledger;
- every named switch is a constant of `Profile`;
- every divergence names its decision, fixture and exemption.

The oracle packets under `davinci/vize_l1/tests/fixtures/htmlish/` hold one
source per row and dialect and the tree its oracle builds (parse5 for
`document`, `@vue/compiler-dom` for `vue`), written by
`tools/commands/davinci/htmlish-oracle.ts` and checked current by the ledger
test. `davinci/vize_l1/tests/htmlish_ledger.rs` parses each source with L1 and
requires the oracle's elements, text, comments and error presence, or a listed
divergence. A packet that starts to match must leave that list.

These divergences are deliberate and are in the ledger, and one design
commitment holds alongside them:

- **Error recovery** (divergence `error-recovery`). Parsing never stops at a
  structural error. Malformed input becomes typed holes (the L1 hole policy),
  so the editor, the linter and the formatter keep working on code being
  typed. An oracle that rejects an input is compared only on inputs it
  accepts.
- **Document refusal** (divergence `document-structure-refusal`). The
  Document structure builder refuses, with a typed reason, a source whose
  browser tree it does not build exactly yet (recovered syntax, `</br>`,
  `plaintext`, `xmp`/`iframe`, `template` contents, script-data escapes). It
  never returns an approximate tree.
- **Vue-template recovery follows HTML** (divergences `vue-implied-end-tags`,
  `comment-bang-close`). In a Vue template, vize closes `p`, `li` and the
  other optional-end elements at the start tags HTML's in-body rules name, as
  the shipped template parser does, and closes a comment at `--!>`. The Vue
  oracle keeps authored nesting and reads a comment to `-->`.
- **Vue-template diagnostics follow HTML** (divergences
  `vue-void-end-tag-reported`, `vue-declaration-reported`). vize reports
  `</br>` and `<!DOCTYPE html>` in a Vue template; the Vue oracle drops both
  silently. The tree is the same.
- **Document is the browser tree** (commitment). The Document profile models real WHATWG
  tree construction: implied end tags, foster parenting, the `select` rules
  and the adoption agency where the structure builder admits them. A Document
  tree is what the browser hydrates, not a token-level approximation.

Owed: an oracle differential fuzzer (generated htmlish inputs run through vize
and both oracles, honoring each divergence's `fuzzExempt`), and real-file
comparisons over the hydrated project corpus. Both remain in step 2d.

### Migration-status names are not concepts

`native` (built natively, not legacy-backed) describes how far a migration has
gone, not what a module is. Under the charter and the #13020 acceptance, it
does not belong in the stable layout, and modules are named by concept instead.
A reader with no migration history must understand every name. `compat` stays
where it names what a module is: a byte-compatible shim of a legacy
implementation (`lex/compat`, the Armature tokenizer), deleted with that
implementation behind its zero-fallback gate (#6854). The levels gate checks
only `native` (maintainer decision 2026-11-17).

| Today (`vize_l1`)                                                                        | Destination                                                                                                                                                                                                    |
| ---------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `markup/htmlish/lex/native.rs`, `lex/native/` (the state machine)                        | the lexer itself: `markup/htmlish/lex.rs` with its states, sequences and character tables directly in `lex/`                                                                                                   |
| `markup/htmlish/lex/compat.rs` (byte-identical Armature tokenizer)                       | stays `lex/compat`, deleted with Armature (#6828)                                                                                                                                                              |
| `markup/htmlish/native/` (typed views over a parsed component template; `Native*` types) | `markup/htmlish/view/`; types renamed by concept (`NativeElement` becomes `ElementView`, …)                                                                                                                    |
| `markup/htmlish/native/{lint,lint_tag}.rs`, `markup/htmlish/surface/lint_tag.rs`         | out of L1. L1 does not know about lint. A view a product needs becomes a neutral read in the L2 query API (#6871); a lint-shaped projection moves to its consumer, `vize_patina` (110 files use `NativeLint*`) |
| `markup/htmlish/event/native.rs`                                                         | `markup/htmlish/event/record.rs` (the event recorder)                                                                                                                                                          |
| `markup/entity/native.rs`                                                                | `markup/entity/decode.rs` (the decoding hook)                                                                                                                                                                  |
| `embed/syntax/for_head/native.rs`                                                        | `embed/syntax/for_head/parse.rs`                                                                                                                                                                               |
| `container/vue/compat/` (the shared SFC block-boundary scanner)                          | `container/vue/boundary/`, its lexical helpers `block/compat` becoming `block/lexical`                                                                                                                         |

The same rule applies in `vize_l1_to_l2` (`native`, `native_file`,
`native_vue2`, `dialect/vue{2,3}/native`, `vue_file/region/native`), `vize_l2`
(`artifact/builder/native`, `file/region/native`, `lang/js/*/native`) and
`vize_l3` (`decision/native`, `decision/{dom,ssr}/…/native`). Those are
renamed by concept in their own steps.

## Target layout

```text
vize_l1/src/
  container/{vue, lang.rs, plain?}                                   lang.rs: block-language registry
  markup.rs  markup/{token.rs, profile.rs, grammar.rs, entity/}   shared contracts
             markup/{htmlish/, pug/, tsrx/}                       one sibling per grammar
  style.rs   style/{structure.rs, structure/, classes.rs, nesting.rs}  shared
             style/{css/, scss.rs, sass.rs, less.rs, stylus.rs}   one sibling per language
  embed/     (JS/TS programs and embeds, parsed once)
  framework/vue/{profile.rs, directive.rs, style_bind.rs, feature/, version/, quirks.rs}
  registry.rs  edit.rs  slice.rs

vize_l1_to_l2/src/
  markup/{htmlish, pug}         grammar → neutral ops
  framework/vue/{sfc, jsx, tsrx, setup?, …}  Vue compositions that need framework meaning
  lang/  pass/  registry.rs     (unchanged roles)

vize_l2/src/
  op/  artifact/  module.rs (ModuleDef, Edit, ComponentDef)  file/ (walk, resolution)
  lang/js  framework/vue/{contract, setup, options, …}  dialect/petite

vize_l2_to_l3/, vize_l3/src/   neutral only: no jsx/, tsrx/, sfc names

vize_l4/src/
  module/   the one assembler: splice, Define/Erase/Access edits, imports, hoists,
            exports, erase.rs, one source map
  targets/{dom, ssr, vapor, ts, vue2}   each emits a ComponentDef's render and shape
  framework/vue  expr/  runtime/  write/  registry.rs
```

## The gate: `levels.inputs`

`tests/tooling/davinci-level-input-placement.test.ts` lands with this record:

- **L2→L3, L3, L4:** no module path segment, file name or Rust identifier
  naming an input (`jsx`, `tsx`, `tsrx`, `sfc`, `pug`, case-insensitive on
  identifier words) outside test files. `lang/` is not an allowed top-level
  module in these crates. Comments and string literals are not code.
- **L1:** the top-level module set is exactly the target set above. `markup/`
  children are the shared contracts plus one directory per grammar, and
  `style/` children are the shared readers plus one module per language.
  The existing `davinci-l1-dialect-isolation` test keeps checking that core
  modules do not reference dialect policy, now against `framework/vue`.
- **Every level crate:** no module path segment, and no declared type,
  function, constant or static, named by migration status (`native`; owed as
  `status:` modules and `ident:` files). In L1, no non-test code names a
  product (`lint` as an identifier word).
- **Fixtures:** every owed row of the Document/SFC difference table is listed
  as a `fixture:` entry until its fixture lands.
- **Ratchet.** Every present difference is listed, one path per entry, in
  `docs/davinci/plan/input-placement-owed.json`. A new difference fails the
  test. A fixed one must leave the list, so the list can only shrink. The
  `fixture:` entries are checked the same way: a row may leave only when its
  fixture exists.
- `levels.inputs` is added to the T3 table of the completion gates as a
  measured gate, by maintainer request (2026-11-17). It passes when the owed
  list is empty.

## Migration

Every structural step is a **scripted move**:
`tools/commands/davinci/level-layout-move.ts <step>` runs `git mv` and rewrites
paths in Rust `mod`/`use` items, `#[path]` attributes, tooling path strings,
storage-policy inventories, pinned source lists and doc links. It makes no
other edit. A sibling branch rebases by re-running the same step on its own
tree. Behavior-preserving edits follow in separate commits. Each PR proves
parity with the existing corpora (compiler differential, mounted parity, the
JSX oracles, the TSRX reference corpus, L1 fidelity), and its PR body carries
`--profile ci-opt` instruction, wall-time and RSS numbers.

| Step | Slice                                                                                                                                                                                                                                                | Kind                        |
| ---: | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------- |
|    1 | This record, the `levels.inputs` gate with its `owed` list, the completion-gates row                                                                                                                                                                 | docs + ratchet              |
|    2 | L1: `pug/` → `markup/pug/`; HTML-family files + `build/` + `event.rs` → `markup/htmlish/`                                                                                                                                                            | move-only                   |
|   2a | L1: `VueComponent` → `framework/vue/profile.rs` (`VueTemplate`)                                                                                                                                                                                      | move + rename               |
|   2b | L1: migration-status names → concept names (`lex/native` → `lex`, `native/` → `view/`, `event/native` → `event/record`, `entity/native` → `entity/decoder`, `for_head/native` → `for_head/parse`, `container/vue/compat` → `container/vue/boundary`) | scripted move               |
|   2c | L1: lint views (`NativeLint*`, `lint_tag`) out of L1, into `vize_patina` or the L2 query API (#6871)                                                                                                                                                 | behavior-preserving         |
|   2d | Owed Document/SFC fixtures; a divergence a new fixture finds is fixed in the same PR                                                                                                                                                                 | fixtures + fixes            |
|    3 | L1: `css/` → `style/` with per-language siblings; `bind.rs` → `framework/vue/style_bind.rs`                                                                                                                                                          | move-only, then split edits |
|    4 | L1: `dialect/vue*` + `markup/directive.rs` → `framework/vue/`; delete `moved.rs`                                                                                                                                                                     | move-only, then removal     |
|    5 | L2: `lang/tsrx` lowering → `vize_l1_to_l2::framework::vue::tsrx`; `component.rs` → `module.rs`                                                                                                                                                       | move-only                   |
|    6 | L2: `ModuleDef`, `Edit`, extended `ComponentDef`; the TSRX lowering fills component facts (port of L4 `component`/`props`/`scopes`/`names`)                                                                                                          | behavior-preserving         |
|    7 | L4: `module/` emits `ModuleDef` on DOM; TSRX switches to it; `lang/tsrx` keeps only the entry, which moves to its product                                                                                                                            | behavior-preserving         |
|    8 | L4/L3: JSX switches to `ModuleDef` (`vize_l4::jsx`, `vize_l3::jsx`, `targets/ts/module.rs` dissolve)                                                                                                                                                 | behavior-preserving         |
|    9 | SFC lowering produces `ModuleDef`; `module/{setup,ordinary}` become edits                                                                                                                                                                            | behavior-preserving         |
|   10 | `ui.jsx-child` → `ui.child-value` rename; `owed` reaches zero                                                                                                                                                                                        | rename-only                 |

Steps 2–4 do not depend on 5–10, and each step is one reviewable PR. Steps 6
and 7 are the ones that stop TSRX code accumulating in L4.

## Coordination

- The TSRX work (davinci-inputs) adds no new code under `vize_l4::lang::tsrx`
  once this record merges. New TSRX component semantics go into the TSRX
  lowering's component facts (step 6), and new emission goes into
  input-neutral `module/` or `targets/*`.
- The style inputs (#11550–#11553) put new code at the `style/` destinations.
  A file added before step 3 lands goes at the target name under `css/`
  (`css/sass.rs`), so that step 3 stays a pure rename.
- Byte-pinned files, such as `crates/vize_canon/src/virtual_ts/generator.rs`,
  are not touched by any step.
