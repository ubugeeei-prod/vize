---
title: Cross-file Complexity
---

# Cross-file Complexity

Vize calculates complexity from retained expression ASTs, lowered template control regions, and
resolved project facts. It does not count tokens in source text or estimate runtime execution time.
The report separates three questions:

| View                  | Question                                                                                   | Used by                                |
| --------------------- | ------------------------------------------------------------------------------------------ | -------------------------------------- |
| **Own template**      | How many decisions does this component's template contain, and how deeply are they nested? | `vue/max-template-complexity`          |
| **Rendered template** | What template complexity is reachable through distinct child components?                   | Doctor's template hotspot notice       |
| **Weighted project**  | How much template and component-boundary data flow does the analyzed project contain?      | Cross-file reports and ranked hotspots |

## Own template: the calculation

Each analyzed template starts at **cyclomatic 1, cognitive 0**. Cyclomatic complexity is
`1 + decisions`; cognitive complexity sums the increments below. A plain `<p>Hello</p>` therefore
scores **1 / 0**, not zero. Script-side branches do not belong to these template scores.

Let `d` be the current nesting depth. A branch body, loop body, or scoped-slot body adds one level;
ordinary elements and components add none. A ternary expression also adds one level while visiting
its test and both alternatives, including nested ternaries in its test.

| Construct                   | Cyclomatic increment                   | Cognitive increment                        | Attributed source      |
| --------------------------- | -------------------------------------- | ------------------------------------------ | ---------------------- |
| `v-if`                      | 1                                      | `1 + d`                                    | Condition              |
| Each `v-else-if`            | 1                                      | 1, regardless of `d`                       | Condition              |
| `v-else`                    | 0                                      | 1, regardless of `d`                       | Branch                 |
| `v-for`                     | 1                                      | `1 + d`                                    | Collection expression  |
| Each `?:`                   | 1                                      | `1 + d`                                    | Conditional expression |
| A logical-operator tree     | Number of `&&`, `\|\|`, `??` operators | Number of runs of the same operator        | Whole logical tree     |
| Scoped slot with parameters | 0                                      | 0; its children have depth `d + 1`         | Slot binding           |
| Unknown expression          | 0                                      | 0; increments the separate unknown counter | Expression             |

Conditions and a loop's collection expression are evaluated at their owner's depth, before entering
the body. A scoped-slot owner's bindings also stay at the owner's depth. Only its children gain the
slot nesting level. Logical runs always have a flat cognitive cost, even inside a deep branch.

For example, three independent `v-if` blocks score **4 / 3**. Nesting those same three conditions
inside one another keeps cyclomatic **4**, but cognitive becomes **1 + 2 + 3 = 6**.
`templateMaxNesting` records the deepest **nonempty template region**, not expression nesting:
`{{ a ? b : c ? d : e }}` scores **3 / 3** with maximum template nesting **0**.

### Logical runs and AST boundaries

A logical tree contains directly connected logical expressions; parentheses are transparent.
Operators are read in source order. A run ends when the operator changes. A call, member access,
TypeScript wrapper, or ternary ends the current tree; logical expressions inside it start separate
trees. These examples contain no surrounding template nesting:

| Interpolation expression | Operators          | Cognitive increment       | Own template score, including the initial 1 |
| ------------------------ | ------------------ | ------------------------- | ------------------------------------------- |
| `a && b && c`            | 2                  | 1                         | 3 / 1                                       |
| `a && b \|\| c && d`     | 3                  | 3                         | 4 / 3                                       |
| `(a \|\| b) ?? c`        | 2                  | 2                         | 3 / 2                                       |
| `a && f(b \|\| c)`       | 2 across two trees | 2                         | 3 / 2                                       |
| `a ? b : c ? d : e`      | 2 ternaries        | `1 + 2` for the ternaries | 3 / 3                                       |

### Evaluated positions, exclusions, and unknowns

The pass visits interpolations, conditions, collection expressions, binding values and dynamic
names, retained event-handler expressions, directive values and dynamic arguments, `.sync`,
`v-memo`, `v-show`, `v-html`, `v-text`, and dynamic slot names. `v-model` counts its read expression
once; its write side refers to the same authored text.

Text, comments, static attributes, loop aliases, and slot parameter binding patterns add no
decisions. Neither do ordinary tag nesting, `v-show` itself, optional chaining (`?.`), logical
assignment (`&&=`, `||=`, `??=`), style-block `v-bind()`, `v-once`, or `v-cloak`. Expressions inside
an otherwise uncounted construct are still visited: `v-show="ready && visible"` scores **2 / 1**.

A slot outlet's fallback adds no decision or nesting **just for being a fallback**. Its contents
are still visited at the existing depth. `<slot><p v-if="ready">Fallback</p></slot>` scores **2 / 1**.
Parent-authored slot content belongs to the parent's own template; a child's implementation does not.

Opaque expressions without a retained JS AST, foreign expressions, and Vue 2 filter chains each
produce an `unknown` row with zero score. Their source text is not guessed. Current handler-body
carriers are not traversed by this expression pass and do not add an unknown row; retained handler
expressions do count. A lowered loop with only an original head scores its loop structure without
walking a collection AST. A low score therefore does not prove unsupported code is simple.
The SFC lint rule skips external templates and non-HTML template languages altogether.

## Worked Bad and Good templates

**Bad for `vue/max-template-complexity`:** this template has own cyclomatic **13** and cognitive
**25**. The example isolates this rule; the referenced data and components are supplied by the app.

```vue
<template>
  <section>
    <h1>{{ user ? user.name : "Guest" }}</h1>
    <DataTable :rows="rows">
      <template #cell="{ row, column }">
        <span v-if="column.key === 'status'" :class="row.active ? 'on' : 'off'">
          {{ row.status ?? "unknown" }}
        </span>
        <a v-else-if="column.key === 'link' && row.url" :href="row.url">{{ row.label }}</a>
        <template v-else>
          <em v-for="tag in row.tags" :key="tag">
            <b v-if="tag.pinned || tag.starred">{{ tag.hot ? "!" : "" }}</b>
          </em>
        </template>
      </template>
    </DataTable>
    <p v-if="!rows.length && !loading">No data</p>
  </section>
</template>
```

The scoped slot contributes no points but puts its children at depth 1. The complete breakdown is:

| Contribution in source order                   | Depth | Cyclomatic | Cognitive |
| ---------------------------------------------- | ----: | ---------: | --------: |
| Initial path                                   |     — |          1 |         0 |
| `user ? user.name : 'Guest'`                   |     0 |          1 |         1 |
| `v-if="column.key === 'status'"`               |     1 |          1 |         2 |
| `row.active ? 'on' : 'off'`                    |     2 |          1 |         3 |
| `row.status ?? 'unknown'`                      |     2 |          1 |         1 |
| `v-else-if="column.key === 'link' && row.url"` |     1 |          1 |         1 |
| That condition's `&&`                          |     1 |          1 |         1 |
| `v-else`                                       |     1 |          0 |         1 |
| `v-for="tag in row.tags"`                      |     2 |          1 |         3 |
| `v-if="tag.pinned \|\| tag.starred"`           |     3 |          1 |         4 |
| That condition's `\|\|`                        |     3 |          1 |         1 |
| `tag.hot ? '!' : ''`                           |     4 |          1 |         5 |
| `v-if="!rows.length && !loading"`              |     0 |          1 |         1 |
| That condition's `&&`                          |     0 |          1 |         1 |
| **Total**                                      |       |     **13** |    **25** |

**Good for this rule:** moving the row's branches into a child leaves a small parent template:

```vue
<template>
  <RowList v-if="ready" :rows="rows" />
</template>
```

Its own score is **2 / 1**: initial path 1, one `v-if` decision, and one unnested cognitive increment.
`RowList` is evaluated separately. Extraction lowers the parent's own score only when the decisions
leave its authored template; keeping the same complex parent-authored slot body keeps those points.
Rendered and weighted project scores may still include the extracted work.

## Rendered template: distinct reachable components

For a root component, start with its own score. Follow resolved **component-usage** edges, collect
distinct reachable file identities, and add each reachable component's own score once. Other import
edges are not render edges. Repeated tags, separate branches, and loop iterations do not multiply
the contribution. Missing template facts add no guessed score; `renderedComponents` still counts
reachable files without facts, so this number alone is not a coverage guarantee.

Assume imports resolve `Page → Grid`, `Page → Card`, and `Grid → Card`. These templates illustrate
the existing rendered-complexity fixture:

```vue
<!-- Page.vue -->
<template>
  <main>
    <Card v-if="hero" :item="hero" />
    <Grid :items="items" />
    <Card :item="footer" />
  </main>
</template>
```

```vue
<!-- Grid.vue -->
<template>
  <ul>
    <li v-for="item in items" :key="item.id"><Card :item="item" /></li>
  </ul>
</template>
```

```vue
<!-- Card.vue -->
<template>
  <article v-if="item.visible">
    <h2>{{ item.title ?? "Untitled" }}</h2>
  </article>
</template>
```

| Component | Own cyclomatic / cognitive | Distinct reachable children | Rendered cyclomatic / cognitive       |
| --------- | -------------------------- | --------------------------- | ------------------------------------- |
| Card      | 3 / 2                      | None                        | 3 / 2                                 |
| Grid      | 2 / 1                      | Card                        | `2 + 3` / `1 + 2` = **5 / 3**         |
| Page      | 2 / 1                      | Grid, Card                  | `2 + 2 + 3` / `1 + 1 + 2` = **7 / 4** |

`Page.renderedComponents` is 2. Card counts once, including its initial path, even though Page uses
it twice and Grid also uses it. A component that renders only itself has rendered = own and zero other
components. In a mutual `Ping ↔ Pong` cycle with own **2 / 1** each, both roots have rendered
**4 / 2**, one other component, and `recursive: true`. The traversal marks recursion only when the
root itself is reachable again; a reachable cycle elsewhere does not make that root recursive.

## Weighted project score: seven dimensions

`complexityReport.cyclomaticScore` and `cognitiveScore` sum **own**, not rendered, template scores
across analyzed components. `dimensions` combines those sums with the other raw facts below.
`totalScore` sums the seven dimension scores; it is not an average or a rendered path count.

| Dimension field       | Exact formula using `complexityReport.input` fields                                                              |
| --------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `templateControlFlow` | `templateCyclomatic + templateCognitive`                                                                         |
| `slotUsage`           | `2 × slotCount + 2 × templateScopedSlotCount`                                                                    |
| `propDrilling`        | `3 × propDrillingEdgeCount`                                                                                      |
| `globalState`         | `2 × globalStateReferenceCount`                                                                                  |
| `provideInject`       | `2 × max(provideInjectMaxDepth − 1, 0) + provideInjectReferenceCount + 2 × max(provideInjectFanoutCount − 1, 0)` |
| `fallthroughAttrs`    | `4 × fallthroughRiskCount`                                                                                       |
| `reactiveGraph`       | `reactiveNodeCount + 2 × reactiveEdgeCount + 10 × reactiveCycleCount`                                            |

The input names describe analyzer counters, with these precise boundaries:

- `slotCount` adds declared slots and slots recorded on component usages. Scoped-slot regions
  also contribute through `templateScopedSlotCount`; neither counter measures runtime calls.
- `propDrillingEdgeCount` counts passed prop entries on component usages, including a single hop.
  It does not count only long prop chains.
- `globalStateReferenceCount` counts existing `ShouldUseStoreToRefs` and `StoreDestructured`
  findings, not every store access or global variable.
- With a provide/inject tree summary, depth is its maximum depth, references are provides plus
  injects, and fan-out is the greater of child fan-out and provider-consumer count. Without that
  summary, references are the number of provide/inject matches; depth and fan-out remain zero.
- `fallthroughRiskCount` adds components with potential issues and risky unconsumed fallthrough
  attributes. Without the summary, it counts components whose fallthrough info has potential issues.
- Reactive nodes are reactive-source registrations, including repeated names; edges and cycles
  come from per-file effect-graph summaries, not arbitrary project import or provide/inject edges.

`componentCount` is the number of registered Vue components; it has no separate weight.
`templateUnknown` sums unknown rows, and `templateMaxNesting` takes the maximum template-region
depth. Neither directly adds points. All count conversions, additions, and weights saturate at
their integer bounds; score arithmetic caps at **4,294,967,295** instead of wrapping.

### Complete numeric example

The existing scoring unit test supplies this input. It is a report-input example, not a claim that
one particular Vue snippet produces all of these project facts:

```json
{
  "componentCount": 1,
  "templateCyclomatic": 6,
  "templateCognitive": 9,
  "templateUnknown": 1,
  "templateMaxNesting": 3,
  "templateScopedSlotCount": 4,
  "slotCount": 3,
  "propDrillingEdgeCount": 2,
  "globalStateReferenceCount": 4,
  "provideInjectMaxDepth": 3,
  "provideInjectReferenceCount": 5,
  "provideInjectFanoutCount": 4,
  "fallthroughRiskCount": 2,
  "reactiveNodeCount": 6,
  "reactiveEdgeCount": 7,
  "reactiveCycleCount": 1
}
```

| Dimension             | Substitution                    | Points |
| --------------------- | ------------------------------- | -----: |
| Template control flow | `6 + 9`                         |     15 |
| Slots                 | `2 × 3 + 2 × 4`                 |     14 |
| Prop drilling         | `3 × 2`                         |      6 |
| Global state          | `2 × 4`                         |      8 |
| Provide/inject        | `2 × (3 − 1) + 5 + 2 × (4 − 1)` |     15 |
| Fallthrough attrs     | `4 × 2`                         |      8 |
| Reactive graph        | `6 + 2 × 7 + 10 × 1`            |     30 |
| **Total**             | `15 + 14 + 6 + 8 + 15 + 8 + 30` | **96** |

The band is `low` for 0–14, `moderate` for 15–34, `high` for 35–69, and `extreme` for 70 and above.
This example is **extreme**, dominated by `reactive-graph` with 30 points. Weighted hotspots reuse
the formula with per-component inputs and rank by total descending, then filename. Their totals
need not sum to the project total: local provide/inject depth and fan-out are attributed separately,
and the formula is nonlinear in those counters.

## Thresholds, locations, and enabling lint

The own-template rule warns strictly above **cyclomatic 11 or cognitive 16**; equality is allowed.
No preset enables it, it has no autofix, and it does not expose a rule-specific threshold option.
These constants are pinned to the p95 of the **2026-09-22** measurement of 40,724 templates; that
historical corpus does not define a project's current score distribution.

Enable it in your [Vite+ configuration](/guide/vite-plus), then run `vp run lint`:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      rules: {
        "vue/max-template-complexity": "warn",
      },
    },
  },
});
```

The warning spans the opening `<template>` tag. It labels at most five nonzero contributors,
ranked by cognitive increment, then cyclomatic increment, then earliest source position; selected
labels are displayed in source order. In the Bad example these are the status `v-if`, the class
ternary, `v-for`, the tag `v-if`, and the inner ternary. Their condition/expression spans identify
the actual work rather than the entire enclosing element.

`templateComplexity` reports file-absolute byte `start` / `end` offsets and 1-based character
`line` / `column` positions for contributors. Components sort by rendered cognitive descending,
then rendered cyclomatic descending, then filename. Doctor emits
`VIZE_DOCTOR_TEMPLATE_COMPLEXITY_HOTSPOT` as a **notice** only above rendered **106 or 139**;
its primary location is the first own contributor, or an empty span at byte 0 when none exists,
and its evidence names up to three leading own contributors. Workspace-unrepresentable paths
are skipped. These rendered limits are independent of the weighted score bands.

The WASM cross-file result exposes `complexityReport`, `complexityHotspots`, and
`templateComplexity`. The weighted report is exploratory; it does not itself fail a build.
An explicitly enabled lint rule follows the configured lint severity policy. Unknown or unavailable
facts remain coverage limits, not evidence that a component is easy to understand.

## Implementation and existing controls

- [Metric specification and historical corpus](https://github.com/ubugeeei-prod/vize/blob/main/docs/davinci/plan/complexity-metrics.md)
- [L2 region and expression calculation](https://github.com/ubugeeei-prod/vize/tree/main/davinci/vize_l1_to_l2/src/pass/cfg)
- [Whole lint example and diagnostic labels](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/max_template_complexity/tests.rs)
- [Cross-file weights, counters, and reachability](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_croquis_cf/src/rules/complexity)
- [Weighted score controls](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/complexity_tests.rs)
