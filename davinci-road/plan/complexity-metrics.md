# Template complexity metrics

**Status:** decided by [P4-9a](./phase-4-tasks-later.md#p4-9a--template-cfg-complexity-facts-and-metric-spec)
(2026-09-22), confirming the [open-question recommendation](../open-questions.md#complexity-metric-definition)
with the amendments listed at the end. Production implementation:
`crates/vize_s1_to_s2/src/pass/cfg.rs` (the `template-complexity` pass).
Independent implementation: `crates/vize_s1_to_s2/tests/cfg_complexity_oracle/`
(TS-34). Cross-file attribution and the rule are
[P4-9b](./phase-4-tasks-later.md#p4-9b--cross-file-complexity-rule-and-doctor-finding).

## Unit and input

The unit is **one component's template**: one S1→S2 lowering, one S2 root
region. Both metrics are computed over the S2 control regions — a `ui.if` owns
one region per branch, a `ui.for` owns its repeated region — and the retained
expression ASTs of the positions the template evaluates. Nothing is scanned
from text.

## Cyclomatic complexity (own)

`cyclomatic = 1 + Σ decisions`. S2 control flow is structured (single entry,
single exit, no jumps), so McCabe's `E − N + 2` over the template's
control-flow graph equals `1 +` the number of binary decisions; the pass
counts decisions and never materializes the graph, and the TS-34 oracle builds
the graph explicitly and measures `E − N + 2`, so their agreement checks the
equivalence rather than one formula twice.

## Cognitive complexity (own)

The SonarSource model applied to templates: `cognitive = Σ increments`, where
a **structure** pays `1 + nesting`, a **hybrid** (`v-else-if`, `v-else`) pays
a flat `1`, and a **run of like logical operators** pays a flat `1`.

**Nesting** is the number of enclosing nesting regions: `ui.if` branch
regions, `ui.for` bodies and scoped-slot bodies; inside one expression, each
enclosing `?:` adds one more. Plain elements and components never nest.
`max-nesting` is the deepest non-empty region depth the template reaches.

## Rules

| Rule          | Construct                                       | Cyclomatic      | Cognitive         | Row span              |
| ------------- | ----------------------------------------------- | --------------- | ----------------- | --------------------- |
| `v-if`        | first branch condition of a `ui.if`             | +1              | +1 + nesting      | the condition         |
| `v-else-if`   | a later branch condition                        | +1              | +1                | the condition         |
| `v-else`      | the unconditional branch                        | 0               | +1                | the branch            |
| `v-for`       | a `ui.for`                                      | +1              | +1 + nesting      | the source expression |
| `scoped-slot` | a carrier with `v-slot` / `slot-scope` params   | 0               | 0 (body nests +1) | the slot binding      |
| `logical`     | a maximal logical-operator tree (see below)     | +1 per operator | +1 per run        | the tree              |
| `conditional` | a `?:`                                          | +1              | +1 + nesting      | the `?:`              |
| `unknown`     | an evaluated position without a retained JS AST | 0 (counted)     | 0 (counted)       | the expression        |

Each `ui.if` therefore contributes "each branch beyond the first, plus one
without a `v-else`" — the same number as one per condition, attributed where
each condition stands.

**Operator trees.** The logical operators are `&&`, `||` and `??`. A tree is
maximal over `LogicalExpression` nodes joined
directly or through parentheses (parentheses are transparent); any other node
— a call, a member access, a TS wrapper, a `?:` — ends the tree, and a logical
expression below it starts its own. Operators are read in source order and a
**run** is a maximal sequence of the same operator: `a && b && c` is one run,
`a && b || c && d` three, `(a || b) ?? c` two.

**Evaluated positions.** `v-if`/`v-else-if` conditions, the `v-for` source
(evaluated once, at the loop's own nesting), interpolations, `v-bind` values
and dynamic names, `v-on` handlers and dynamic names, the `v-model` read (its
write side is the same authored text and is not counted twice), directive
values and dynamic arguments, `.sync`, `v-memo`, `v-show`, `v-html`, `v-text`,
dynamic `v-slot` names, and a slot outlet's dynamic name and bindings. An
owner's bindings evaluate at the owner's own depth; only its children stand
inside a scoped slot.

**Not counted.** `v-for` aliases and slot params (binding patterns, not
evaluated code); a slot outlet's fallback (the parent's choice, not this
template's — it neither counts nor nests); `v-show` itself (both states
render the same subtree); `?.`; logical assignment (`&&=`, `||=`, `??=`);
style-block `v-bind()`; `v-once` and `v-cloak` (no expression).

**Unknown.** An opaque expression (no retained AST: multi-statement handlers,
text the parser or nesting guard refused), a foreign-dialect expression and a
Vue 2 filter chain add nothing and are recorded as one `unknown` row each, so
"simple" and "not analysed" stay distinguishable (the opaque pessimal law: no
conclusion is drawn from the text).

## Corpus distribution and default thresholds

Measured 2026-09-22 at `origin/main` `8395072f1` over the **full corpus**
(all 146 `tests/_fixtures/_git` submodules at their pinned commits): 41,580
`.vue` files, 40,724 templates scored (357 without a template, 499 in a
non-HTML template dialect), 124,732 breakdown rows, 10,436 `unknown`
positions, and exact production/naive agreement on every template.

| metric     |      n | p50 | p90 | p95 | p99 |  max |
| ---------- | -----: | --: | --: | --: | --: | ---: |
| cyclomatic | 40,724 |   1 |   7 |  11 |  27 |  528 |
| cognitive  | 40,724 |   0 |   9 |  16 |  48 | 1577 |

Percentiles are nearest-rank. The command that reproduces the table:

```sh
git submodule update --init --depth 1 tests/_fixtures/_git
VIZE_DAVINCI_COMPLEXITY_CORPUS=tests/_fixtures/_git \
  cargo test -p vize_s1_to_s2 --test cfg_complexity_oracle \
  the_corpus_shard_agrees_with_the_naive_evaluator -- --exact --nocapture
```

**Default thresholds, pinned at the recorded p95 (warning):** a component
whose own **cyclomatic complexity exceeds 11** or whose own **cognitive
complexity exceeds 16** warns. Strictly above p95 means at most one component
in twenty warns on an unconfigured project. A re-measurement that moves p95
re-pins these numbers in the same change as the table.

## Attribution (P4-9b)

A child never taxes its parent's **own** score: the lint rule reads own
complexity only, so extracting a component never makes the parent look worse.
**Rendered** complexity — own plus Σ own over the distinct child components
reachable in the render tree, strongly connected components collapsed so a
recursive component counts once — is the cross-file number and drives Doctor
hotspots.

The render tree follows the analyzer's component-usage edges, which resolve
each tag through the importing file's own bindings first (so an aliased import
reaches the imported file). Reachability is a set: a child rendered from two
branches or two parents counts once per root, and a component that reaches
itself (directly or through mutual recursion) is marked `recursive` and adds
nothing further.

### Rendered distribution and the Doctor hotspot threshold

Measured 2026-09-22 over the same full corpus, one analyzer per submodule (142
projects with `.vue` files, 40,724 components, 70,700 render edges, 695
recursive components). The run also checks two laws on every component:
rendered ≥ own (equal when nothing is rendered), and rendered never shrinks
from a child to its parent.

| metric              |      n | p50 | p90 | p95 | p99 |   max |
| ------------------- | -----: | --: | --: | --: | --: | ----: |
| rendered cyclomatic | 40,724 |   4 |  52 | 106 | 355 |  6814 |
| rendered cognitive  | 40,724 |   2 |  60 | 139 | 526 | 13778 |
| rendered components | 40,724 |   1 |  12 |  24 |  56 |   371 |

```sh
VIZE_DAVINCI_RENDERED_CORPUS=tests/_fixtures/_git \
  cargo test -p vize_curator --lib the_rendered_distribution_over_a_corpus_obeys_its_laws \
  -- --nocapture
```

**Doctor hotspot, pinned at the recorded p95:** a component whose rendered
cyclomatic complexity exceeds 106 or whose rendered cognitive complexity
exceeds 139 gets one `VIZE_DOCTOR_TEMPLATE_COMPLEXITY_HOTSPOT` notice
(maintainability). Its invalidation inputs are every file in the render tree.

## Amendments to the recommendation

1. **Scoped slots nest; they do not count.** A scoped-slot body is a render
   callback — the lambda case of the cognitive model — so it deepens nesting
   for everything inside it and adds no increment of its own (nor a
   cyclomatic decision).
2. **`v-else-if` and `v-else` are hybrids** (a flat +1 each, no nesting
   penalty), as Sonar's `else if` / `else`.
3. **`unknown` covers every non-JS payload**, not only `Opaque`: foreign
   dialects and Vue 2 filter chains are not analysed either.
4. **Handlers count.** A `v-on` handler is evaluated template code; its
   operators and `?:` count like any other expression's.
5. **Thresholds warn strictly above p95** (the recommendation said "at p95").
