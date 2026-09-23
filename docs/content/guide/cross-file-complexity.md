---
title: Cross-file Complexity
---

# Cross-file Complexity

Vize's cross-file complexity report is a project-graph summary produced by Croquis. It is not a
diagnostic rule by itself; it is an explainable score that downstream tools can show in reports,
Playground, and future threshold-based checks.

The model maps three complexity signals to Vue:

- Template path count: each component's own cyclomatic complexity, computed by
  Davinci's S2 `template-complexity` analysis. It counts every `v-if` /
  `v-else-if` condition, every `v-for`, and every `&&`, `||`, `??` and `?:` in
  the expressions the template evaluates.
- Nested control flow: each component's own cognitive complexity. Branches and
  loops cost more the deeper they are nested inside `v-if`, `v-for` and
  scoped-slot regions.
- Component-boundary data flow: props, provide/inject, and reactive edges remain
  visible as cross-boundary signals instead of being flattened into one file.

The metric definition and its corpus-pinned thresholds live in
[`complexity-metrics.md`](https://github.com/ubugeeei-prod/vize/blob/main/docs/davinci/plan/complexity-metrics.md).

## Scores

The report exposes both raw signals and derived scores.

| Field             | Meaning                                                                                                                            |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `cyclomaticScore` | Sum of every component's own template cyclomatic complexity.                                                                       |
| `cognitiveScore`  | Sum of every component's own template cognitive complexity.                                                                        |
| `totalScore`      | Sum of dimension scores: template flow, slots, prop drilling, global state, provide/inject, fallthrough attrs, and reactive graph. |
| `band`            | Human-facing bucket: `low`, `moderate`, `high`, or `extreme`.                                                                      |

The raw input also keeps the numbers behind the score, including:

| Signal                                                             | Why it matters                                                                                                 |
| ------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------- |
| `templateCyclomatic` and `templateCognitive`                       | The own template scores summed across components.                                                              |
| `templateMaxNesting`                                               | The deepest nesting of branches, loops and scoped slots inside a single template.                               |
| `templateScopedSlotCount`                                          | Scoped slots couple parent and child templates, so they are counted separately from ordinary slots.           |
| `templateUnknown`                                                  | Expressions without a parsed AST (multi-statement handlers, for example). They add nothing to either score.   |
| `propDrillingEdgeCount`                                            | Prop edges indicate cross-boundary data flow.                                                                  |
| `provideInjectMaxDepth` and `provideInjectReferenceCount`          | Deep or broad DI trees make ownership harder to inspect locally.                                               |
| `reactiveNodeCount`, `reactiveEdgeCount`, and `reactiveCycleCount` | Reactive graphs capture declaration-level state, effects, and loss-prone cycles.                               |

## Component Boundaries

Template complexity has two views, and both come from the same facts:

- **Own** complexity is the component's template alone. The
  `vue/max-template-complexity` lint rule judges this view, so extracting a branch into a child
  component always lowers the parent's score.
- **Rendered** complexity is the component's own score plus the own score of every distinct
  component it renders, following the component-usage graph that Croquis resolves through imports.
  A child rendered from two places counts once. A recursive component, and a group of components
  that render each other, also count once.

`CrossFileResult.templateComplexity` lists every component with both views, most complex render
tree first. For each component it also gives the constructs that add complexity, with line and
column.

This means a shallow-looking component can still produce a high score when it forwards scoped slots,
drills props, or depends on a deep provide/inject path. The Playground's Cross-file mode shows the
score beside diagnostics so those signals are visible while editing fixtures.

## Lint Rule and Doctor Finding

`vue/max-template-complexity` reports a `warning` when a component's own template has cyclomatic
complexity above 11 or cognitive complexity above 16. Those limits are the p95 over Vize's
real-world corpus of 40,724 templates. The warning points at the `<template>` tag and labels the
five constructs that add the most complexity.

Because the limits are a p95, about one real component in twenty exceeds them. No preset enables
the rule, so turning it on is a project decision. Name it under `linter.rules` to enable it:

```ts
export default defineConfig({
  linter: {
    rules: {
      "vue/max-template-complexity": "warn",
    },
  },
});
```

`vize doctor` reports a template-complexity hotspot, as a notice, when a component's rendered
complexity is above the corpus p95: cyclomatic 106 or cognitive 139.

Cyclomatic complexity adds 1 for each decision: every `v-if` / `v-else-if` condition, every
`v-for`, and every logical operator and `?:`. Cognitive complexity counts as follows:

- `v-if` and `v-for` add 1 plus their nesting depth.
- `v-else-if` and `v-else` add 1 each.
- Each run of `&&`, `||` or `??` adds 1.
- A `?:` adds 1 plus its nesting depth.
- A scoped-slot body counts as one level deeper.

## Hotspots

The report also exposes ranked hotspots so tools can point to the files/components that create the
score instead of showing only one project-level number. Each hotspot carries the local score input,
dimension scores, total score, and dominant dimension. Use `dominantDimension` to explain why the
entry is high, then use `input` to show the raw signal that drove it.

## Current Surface

The public JSON shape is available from the WASM cross-file binding as
`CrossFileResult.complexityReport`, `CrossFileResult.complexityHotspots` and
`CrossFileResult.templateComplexity`. The CLI does not fail
builds on this score yet. Use the report as an exploratory signal, then promote stable thresholds
only after project-specific baselines exist.
