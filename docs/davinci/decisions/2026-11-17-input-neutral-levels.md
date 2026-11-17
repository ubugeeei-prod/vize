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
   grammar or language and one Vue owner. The target trees are below. This
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
| `markup/{lex,parse,document,native,surface,render}`, root `build.rs` + `build/`, `event.rs` | `markup/html/` (the HTML-family template grammar, document and component profiles)                                                                                                                                                                                                      |
| `markup/{token,profile,grammar}.rs`, `markup/entity`                                        | stay at the `markup/` root: contracts and decoding shared by every grammar                                                                                                                                                                                                              |
| `pug.rs`, `pug/`                                                                            | `markup/pug/`                                                                                                                                                                                                                                                                           |
| `markup/tsrx/`                                                                              | stays (`markup/tsrx/`), now a sibling of `html` and `pug`                                                                                                                                                                                                                               |
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

## Target layout

```text
vize_l1/src/
  container/{vue, plain?}
  markup.rs  markup/{token.rs, profile.rs, grammar.rs, entity/}   shared contracts
             markup/{html/, pug/, tsrx/}                          one sibling per grammar
  style.rs   style/{structure.rs, structure/, classes.rs, nesting.rs}  shared
             style/{css/, scss.rs, sass.rs, less.rs, stylus.rs}   one sibling per language
  embed/     (JS/TS programs and embeds, parsed once)
  framework/vue/{directive.rs, style_bind.rs, feature/, version/, quirks.rs}
  registry.rs  edit.rs  slice.rs

vize_l1_to_l2/src/
  markup/{html, pug}            grammar → neutral ops
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
- **Ratchet.** Every present difference is listed, one path per entry, in
  `docs/davinci/plan/input-placement-owed.json`: 72 at this record (13 L1
  layout entries, 59 L2→L3/L3/L4 files). A new difference fails the test. A
  fixed one must leave the list, so the list can only shrink.
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

| Step | Slice                                                                                                                                       | Kind                        |
| ---: | ------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------- |
|    1 | This record, the `levels.inputs` gate with its `owed` list, the completion-gates row                                                        | docs + ratchet              |
|    2 | L1: `pug/` → `markup/pug/`; HTML-family files + `build/` + `event.rs` → `markup/html/`                                                      | move-only                   |
|    3 | L1: `css/` → `style/` with per-language siblings; `bind.rs` → `framework/vue/style_bind.rs`                                                 | move-only, then split edits |
|    4 | L1: `dialect/vue*` + `markup/directive.rs` → `framework/vue/`; delete `moved.rs`                                                            | move-only, then removal     |
|    5 | L2: `lang/tsrx` lowering → `vize_l1_to_l2::framework::vue::tsrx`; `component.rs` → `module.rs`                                              | move-only                   |
|    6 | L2: `ModuleDef`, `Edit`, extended `ComponentDef`; the TSRX lowering fills component facts (port of L4 `component`/`props`/`scopes`/`names`) | behavior-preserving         |
|    7 | L4: `module/` emits `ModuleDef` on DOM; TSRX switches to it; `lang/tsrx` keeps only the entry, which moves to its product                   | behavior-preserving         |
|    8 | L4/L3: JSX switches to `ModuleDef` (`vize_l4::jsx`, `vize_l3::jsx`, `targets/ts/module.rs` dissolve)                                        | behavior-preserving         |
|    9 | SFC lowering produces `ModuleDef`; `module/{setup,ordinary}` become edits                                                                   | behavior-preserving         |
|   10 | `ui.jsx-child` → `ui.child-value` rename; `owed` reaches zero                                                                               | rename-only                 |

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
