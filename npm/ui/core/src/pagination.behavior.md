# Pagination Behavior Contract

Normative state x input -> outcome table for `pagination.vue`,
`pagination-list.vue`, `pagination-item.vue`, `pagination-page.vue`,
`pagination-previous.vue`, `pagination-next.vue`, and
`pagination-ellipsis.vue` (`@vizejs/ui/pagination`). Every row is proven by
focused tests plus package export, family catalog, renderer, runtime
conformance, and size-budget gates.

| ID  | State                  | Input                 | Outcome                                                                                   | Evidence                                                                      |
| --- | ---------------------- | --------------------- | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| P1  | default / current page | render                | root, list, controls, page items, ellipses, slots, deterministic ids, ARIA, and data land | `renders accessible pagination semantics with deterministic ids and range`    |
| P2  | uncontrolled           | page / next click     | internal page changes, `update:modelValue` and `change` emit once per distinct request    | `clicks update uncontrolled page state and suppress current-page repeats`     |
| P3  | controlled             | page click            | emits requested page while rendered current page waits until the parent accepts it        | `controlled page wins until the parent accepts the request`                   |
| P4  | disabled / boundary    | click, Space, or Tab  | unavailable controls are native-disabled, leave tab order, and do not emit page changes   | `disabled roots and boundary controls suppress activation and tab focus`      |
| P5  | exposed instances      | focus, setPage, reset | public refs expose live state and imperative focus/page methods                           | `exposes typed state and imperative page controls`                            |
| P6  | missing provider       | setup                 | compound parts fail closed with the shared context diagnostic                             | `compound parts require a matching root provider`                             |
| P7  | range helper           | compact range render  | boundary/sibling windows are deterministic and one-page gaps expand without ellipses      | `renders accessible pagination semantics with deterministic ids and range`    |
| P8  | out-of-range page      | render                | invalid page controls are disabled and receive non-colliding deterministic ids            | `out-of-range page controls stay disabled without duplicating valid page ids` |
| P9  | SSR and hydration      | isolated render/mount | generated ids are byte-identical per request and hydrate without replacement warnings     | `pagination-ssr.test.ts`                                                      |

The primitive renders no styling. `PaginationPage`, `PaginationPrevious`, and
`PaginationNext` are native buttons so keyboard activation follows the platform.
`PaginationPage` uses `aria-current="page"` for the current page while keeping
the current page focusable; root-disabled, boundary, and out-of-range controls
use native `disabled`.
