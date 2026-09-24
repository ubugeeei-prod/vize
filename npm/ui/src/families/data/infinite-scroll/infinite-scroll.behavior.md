# InfiniteScroll Behavior Contract

Normative state x input -> outcome table for `infinite-scroll-root.vue`,
`infinite-scroll-sentinel.vue`, `infinite-scroll-load-more.vue`,
`infinite-scroll-status.vue`, and `infinite-scroll-item.vue`
(`@vizejs/ui/infinite-scroll`). Every row is proven by the named test.

The loading state is `disabled > loading > complete > error > idle` in priority
order. A request is accepted only while `idle`, or while `error` from the
load-more button or the imperative API, so a failing page never loops through
the sentinel. `loader` promises drive `loading` automatically; consumers that
fetch through the `loadMore` emit own the busy flag through the `loading` prop.

| ID   | State                | Input                           | Outcome                                                                                                                               | Evidence                                                                            |
| ---- | -------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| IS1  | idle                 | render                          | root carries a deterministic id, `aria-busy="false"`, feed semantics when `feed`, observed hidden sentinel, wired button, live status | `renders sentinel, load-more, status, and feed semantics with deterministic wiring` |
| IS2  | idle                 | sentinel intersects             | emits `loadMore("sentinel")` once, runs `loader`, and stays `loading` (`aria-busy="true"`, button disabled) until the promise settles | `the sentinel requests once per intersection and the loader promise drives loading` |
| IS3  | loading -> idle      | loader resolves                 | returns to `idle` and re-observes the sentinel, so a still-visible sentinel requests the next page                                    | `the sentinel requests once per intersection and the loader promise drives loading` |
| IS4  | controlled `loading` | prop changes / `hasMore=false`  | the parent owns the busy flag; `complete` hides and disables the button and ignores the sentinel                                      | `controlled loading and hasMore settle state without a loader`                      |
| IS5  | loading              | loader rejects or throws        | moves to `error`, emits `error(reason, trigger)`, exposes the reason, and ignores the sentinel until the button or API retries        | `loader failures stop automatic loading until an explicit retry`                    |
| IS6  | idle                 | button click                    | emits `click` first; `preventDefault()` keeps state unchanged, otherwise requests with trigger `button`                               | `the load-more button requests pages and honors preventDefault`                     |
| IS7  | disabled             | intersection / click / API      | every request is refused                                                                                                              | `disabled roots suppress every request`                                             |
| IS8  | `scrollRoot="self"`  | mount / `refresh()`             | the sentinel observes against the root element with `rootMargin`; `refresh()` re-observes                                             | `self scroll roots observe the sentinel against the root element`                   |
| IS9  | sentinel             | intersection batches            | exposes the latest intersection state and ignores empty batches                                                                       | `the sentinel exposes its intersection state`                                       |
| IS10 | feed                 | PageDown / PageUp               | moves focus between articles; PageDown on the last article requests the next page; other keys and non-feed roots are untouched        | `feed PageDown and PageUp move focus between articles and request at the end`       |
| IS11 | exposed instance     | read / `loadMore()` / `retry()` | exposes state, busy, hasMore, error, element; `retry()` requires `error`; unknown totals announce `aria-setsize="-1"`                 | `exposes typed state and imperative paging controls`                                |
| IS12 | missing provider     | setup                           | compound parts fail closed with the shared context diagnostic                                                                         | `compound parts require a matching root provider`                                   |
| IS13 | SSR                  | isolated requests               | markup is byte-identical, including generated ids, feed positions, and the live region                                                | `renders byte-identical infinite scroll markup across isolated SSR requests`        |
| IS14 | SSR / hydration      | hydrate                         | server markup hydrates without warnings or node replacement                                                                           | `hydrates infinite scroll markup without warnings or node replacement`              |
| IS15 | types                | compile                         | states, triggers, observer roots, slot state, and exposes are closed and read-only                                                    | `infinite-scroll.types.test-d.ts`                                                   |

Without `IntersectionObserver` the sentinel stays inert and the load-more button
remains the paging control. The intersection root and margin are read when the
sentinel mounts. Control+Home/End feed shortcuts are optional in the APG pattern
and are not implemented.
