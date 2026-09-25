# Pager behavior contract

Normative state x input -> outcome table for `pager.vue`, `pager-tab-list.vue`,
`pager-tab.vue`, `pager-viewport.vue`, and `pager-page.vue`
(`@vizejs/ui/pager`): a swipeable, scroll-snap page view whose segmented tab bar
follows the WAI-ARIA APG tabs pattern with automatic activation. `pages` infers
the page-id union. Minimal consumer CSS: the viewport is `display: flex;
overflow-x: auto; scroll-snap-type: x mandatory` and pages are `flex: 0 0 100%;
scroll-snap-align: start`. Every row is proven by the named test in
`pager.test.ts` or `pager-ssr.test.ts`.

| #   | State                       | Input                           | Outcome                                                                                                                                            | Proven by                                                                                   |
| --- | --------------------------- | ------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| PG1 | seeded                      | render                          | labelled tablist; selected tab is the roving tab stop wired to its tabpanel; off-screen panels are `inert`                                         | `renders APG tabs wired to scroll-snap pages with inert off-screen panels`                  |
| PG2 | any                         | tab click                       | the page becomes active, the viewport smooth-scrolls to it, and `change(page, previous, "tab")` fires                                              | `selecting a tab scrolls the viewport smoothly and emits typed changes`                     |
| PG3 | focused tablist             | Arrow / Home / End              | arrows (RTL-aware) wrap, Home/End jump; focus and selection move together                                                                          | `arrow keys, Home, and End move the selected segment with wrapping`                         |
| PG4 | swiping                     | scroll quiet period / scrollend | the page nearest the scroll position becomes active with reason `scroll`; programmatic scrolls never re-select; `next`/`previous` stop at the ends | `user swipes settle on the nearest page while programmatic scrolls are ignored`             |
| PG5 | controlled / reduced motion | parent change                   | controlled values win and parent changes scroll the viewport; reduced motion scrolls with `behavior: "auto"`                                       | `controlled pages scroll when the parent changes them and reduced motion scrolls instantly` |
| PG6 | invalid setup               | empty `pages` / missing root    | throws `VIZE_UI_PAGER_PAGES` or `VIZE_UI_CONTEXT_MISSING: Pager`                                                                                   | `rejects empty page lists and parts outside a Pager`                                        |
| PG7 | SSR / hydration             | isolated requests               | byte-identical tabs and panels; the initial page is scrolled into place only after hydration                                                       | `renders byte-identical tabs and pages and hydrates without mismatches`                     |

The subpath ships no CSS.
