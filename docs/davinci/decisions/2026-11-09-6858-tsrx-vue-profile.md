# TSRX: Vue host profile, reference oracle and level placement

Issue: #6858 (acceptance checklist: #6252). Builds on the neutral designs of
#6855 ([L1 axes](./2026-10-04-l1-neutral-axes.md)), #6856
([L2 vocabulary](./2026-10-04-l2-neutral-vocabulary.md)) and #6857
([L3 vocabulary](./2026-10-04-l3-neutral-vocabulary.md)). This record makes the
TSRX-specific choices those designs left open. It claims no implementation;
the inventory of the reference tests is in the
[companion record](./2026-11-09-6858-tsrx-reference-inventory.md).

## Profile

TSRX separates core syntax from host semantics
([specification](https://tsrx.dev/specification), draft of 2026-06-07). Vize
implements core TSRX syntax and the **Vue host profile only**. Syntax that is
accepted is never a claim about React, Preact, Solid, Hono, Ripple or Octane
semantics.

The Vue profile is defined by the observable behavior of the reference Vue
pipeline, pinned exactly:

| Role                     | Package             | Version     | Source                                                                   |
| ------------------------ | ------------------- | ----------- | ------------------------------------------------------------------------ |
| Parser and analyzer      | `@tsrx/core`        | 0.5.5       | `tsrx-org/tsrx` `packages/tsrx`                                          |
| Vue code generation      | `@tsrx/vue`         | 0.3.5       | tag `@tsrx/vue@0.3.5`, commit `b09820a9b6b6ed44e640be8cca89b3debbb23491` |
| Vue runtime helpers      | `@tsrx/vue-runtime` | 0.1.13      | dependency of `@tsrx/vue` 0.3.5                                          |
| JSX to Vapor compilation | `vue-jsx-vapor`     | 3.2.26      | existing `jsx-vapor-oracle` catalog                                      |
| Runtime                  | `vue`               | 3.6.0-rc.10 | existing `jsx-vapor-oracle` catalog                                      |

The `tsrx-oracle` catalog in `pnpm-workspace.yaml` pins the first two; the
private `tests/tooling/tsrx-oracle` package uses them only as a dev-time
differential oracle. Products never depend on it. A version bump is a
deliberate change that re-records the committed reference corpus.

Host-defined points follow the reference: a function whose body is a statement
container (`@{ … }`) or that returns a TSRX value, and whose static name starts
with an uppercase letter, is a Vue component (`defineVaporComponent` in the
reference). Its setup statements run once per instance and its output node is
the component's render. Component-level `await` is unsupported, as in the
reference.

## What parity means

The reference emits TSX for `vue-jsx-vapor`, which then emits Vapor code. Vize
emits through its own L4 targets, so byte parity with the reference is not a
meaningful contract. Parity is **behavioral and diagnostic**:

- Mounted parity: for each corpus row, the vize output and the reference
  output are mounted in the same pinned runtime. The normalized DOM
  observation (as in the existing `jsx-vapor-oracle`) must be equal after the
  initial mount and after every scripted state change; event handlers, retained
  node identity across updates and unmount cleanup are observed the same way.
- Diagnostic parity: for each error row, vize reports the same TSRX code
  (`TSRX1xxx` syntax, `TSRX2xxx` template, `TSRX3xxx` style, `TSRX4xxx`
  other; TypeScript-shared mistakes keep their `TS` code) at the same span.
- A row that cannot reach parity yet is an expected failure with a linked
  issue, never a deleted row.

There is no legacy TSRX lane, so no legacy route can satisfy a TSRX row.

## Level placement

TSRX is markup embedded in script: the reverse of a Vue template's embedded
expressions. Under the #6855 axes it is the composition: container `plain`,
markup `tsrx` inside the retained host-language AST, profile `component`,
language TypeScript, framework feature composition TSRX with the Vue profile.

### L1: one parse, exact spans

A `.tsrx` module is parsed once by the existing oxc-based parser
(`upstream/forks/oxc_parser`). The fork gains a framework-neutral markup hook:
where the host grammar reaches an expression start that opens embedded markup
(`<` in a JSX-enabled profile, or `@` followed by `{`, `if`, `for`, `switch` or
`try`), or a function body written as `@{`, it hands the position to a
registered markup grammar. The grammar parses the region and returns its end;
the host keeps an empty placeholder node with the region's exact span. While
parsing the region, the grammar asks the same parser instance, through a
cursor, to parse every embedded expression, statement, binding and head. Those
nodes are ordinary oxc AST nodes in the same arena with original spans; there
is no wrapper offset, no second parse and no parallel identifier scanner.

The TSRX grammar itself lives in `vize_l1::markup::tsrx`: elements and
fragments, dynamic tags `<{expr}>`, attributes (string, expression,
`{name}` shorthand, spread), JSX text and entities, comments between
children, raw `<style>` and `<script>` bodies, statement containers and the
`@if`/`@for`/`@switch`/`@try` directives. The module is the only place that
knows TSRX syntax. The line-start `<` rule of the reference (a `<` that starts a
line after a complete value starts markup, not a comparison) is the one
host-grammar change, and it applies only while a markup grammar is
registered.

### L2: neutral ops first

Elements, components, text, expression children, `@if` and keyed `@for` lower
into the existing neutral ops. Bindings from `@{ … }` setup statements,
`@for` heads and `@catch` parameters use the existing lexical-scope and
reference resolution over the retained AST; a TSRX region opens a nested
scope exactly where the reference does.

IR extensions are limited to what existing ops cannot represent, each landing
with the lowering that needs it:

- `@switch` evaluates its discriminant once and compares with strict equality
  without fall-through. Lowering it into a chain of `ui.if` tests would
  evaluate the discriminant per branch, so it needs either a switch region op
  or an explicit single-evaluation temporary.
- `@for … @empty` needs an empty branch on the iteration op.
- `@try`/`@pending`/`@catch` needs a boundary op with explicit pending, error
  and reset lifetimes. Until it lands, `@try` is a typed unsupported-profile
  diagnostic.

### L3 and L4

DOM is emitted first, then SSR and Vapor, all through `vize_l4` targets and the
shared L3 decisions. TSRX adds no target of its own.

## Explicitly unsupported until implemented

These constructs are parsed (so spans and formatting stay exact) but produce an
unsupported-profile diagnostic at their span instead of being erased:

- `@try` boundaries (until the boundary op lands);
- sibling-scoped `<style>` blocks, themes and `apply` (until the TSRX style
  scoping lands; they are not Vue SFC scoped CSS and must not be lowered as
  such);
- `module` server submodules and `import … from server` (host-defined
  transport; the Vue profile has no isolation contract yet);
- compile-time platform flags (`import.meta.env.platform.*`).

## Delivery

Small PRs, each with tests, keep #6858 open until its whole body and the #6252
checklist are satisfied: this record and the pinned oracle with its reference
corpus; the parser hook and the L1 grammar with span tests; L2 lowering; DOM
emit with mounted parity rows; SSR and Vapor; CLI and bundler integration of
`.tsrx`; type-check and LSP projection (coordinated with the owners of the
content mapper and the virtual TS emitters); documentation.
