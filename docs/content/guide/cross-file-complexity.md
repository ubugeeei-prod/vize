---
title: Cross-file Complexity
---

# Cross-file Complexity

Vize's cross-file complexity report is a project-graph summary produced by Croquis. It is not a
diagnostic rule by itself; it is an explainable score that downstream tools can show in reports,
Playground, and future threshold-based checks.

The model maps three complexity signals to Vue:

- Template path count: one base point per component, plus `v-if`, `v-for`, and
  boolean operators in `v-if` expressions.
- Nested control flow: deeper template flow costs more, including nesting that
  continues through child components.
- Component-boundary data flow: props, provide/inject, and reactive edges remain
  visible as cross-boundary signals instead of being flattened into one file.

## Scores

The report exposes both raw signals and derived scores.

| Field             | Meaning                                                                                                                            |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `cyclomaticScore` | Component base count + `v-if` + `v-for` + boolean operators in `v-if`.                                                             |
| `cognitiveScore`  | Component-tree template nesting score across `v-if`, `v-for`, and scoped slots.                                                    |
| `totalScore`      | Sum of dimension scores: template flow, slots, prop drilling, global state, provide/inject, fallthrough attrs, and reactive graph. |
| `band`            | Human-facing bucket: `low`, `moderate`, `high`, or `extreme`.                                                                      |

The raw input also keeps the numbers behind the score, including:

| Signal                                                             | Why it matters                                                                                                 |
| ------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------- |
| `componentTreeVIfMaxDepth`                                         | Long conditional paths across parent and child components need more states to test.                            |
| `componentTreeVForMaxDepth`                                        | Loops nested across component boundaries amplify render and data-shape complexity.                             |
| `componentTreeScopedSlotMaxDepth`                                  | Scoped slots couple parent and child templates, so their depth is tracked separately from ordinary slot count. |
| `propDrillingEdgeCount`                                            | Prop edges indicate cross-boundary data flow.                                                                  |
| `provideInjectMaxDepth` and `provideInjectReferenceCount`          | Deep or broad DI trees make ownership harder to inspect locally.                                               |
| `reactiveNodeCount`, `reactiveEdgeCount`, and `reactiveCycleCount` | Reactive graphs capture declaration-level state, effects, and loss-prone cycles.                               |

## Component Boundaries

Template complexity is not limited to one SFC. Croquis builds a module registry and component-usage
graph first, then walks component edges with cycle protection. A parent `v-if` around a child, a
parent `v-for` around a child, and a child scoped slot all contribute to the same component-tree
nesting path.

This means a shallow-looking component can still produce a high score when it forwards scoped slots,
drills props, or depends on a deep provide/inject path. The Playground's Cross-file mode shows the
score beside diagnostics so those signals are visible while editing fixtures.

## Current Surface

The public JSON shape is available from the WASM cross-file binding as
`CrossFileResult.complexityReport`. The CLI does not fail builds on this score yet. Use the report as
an exploratory signal, then promote stable thresholds only after project-specific baselines exist.
