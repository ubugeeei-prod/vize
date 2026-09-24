# Table of Contents Behavior Contract

Normative state x input -> outcome table for `toc-root.vue`, `toc-list.vue`,
`toc-item.vue`, and `toc-link.vue` (`@vizejs/ui/toc`), plus
`collectTocEntries`. Every row is proven by the named test in `toc.test.ts` or
`toc-ssr.test.ts`; compile-only guarantees live in `toc.types.test-d.ts`.

The root is a labelled `<nav>` landmark with nested `<ol>` lists of native
fragment links. The link whose section is in view carries
`aria-current="location"`. Active tracking reuses the `@vizejs/ui/scroll-spy`
composable over the ids of the registered links.

| ID  | State                      | Input                   | Outcome                                                                                     | Evidence                                                                  |
| --- | -------------------------- | ----------------------- | ------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| C1  | default                    | render                  | labelled `nav`, nested `ol` lists, `#fragment` links, `aria-current` on the default id      | `renders a labelled navigation landmark with fragment links`              |
| C2  | tracking                   | scroll                  | the section past the offset line becomes active on its link and item                        | `tracks the section in view and marks links and items active`             |
| C3  | controlled / `track=false` | scroll, prop change     | the parent owns `activeId`; no tracking updates are emitted                                 | `controlled activeId and disabled tracking leave ownership to the parent` |
| C4  | `scrollBehavior` set       | primary click, modified | smooth script scrolling, immediate activation, fragment update; modified clicks stay native | `scrollBehavior drives smooth navigation and updates the fragment`        |
| C5  | no `scrollBehavior`        | click                   | native fragment navigation is not intercepted                                               | `native fragment navigation is untouched without scrollBehavior`          |
| C6  | exposed root               | scrollTo, refresh       | imperative navigation and re-measurement; `ariaLabelledby` replaces the default label       | `exposes scrollTo and refresh`                                            |
| C7  | rendered content           | `collectTocEntries`     | id-bearing headings in document order with normalized text and levels                       | `collectTocEntries reads headings with ids in document order`             |
| C8  | missing provider           | setup                   | links and items fail closed with the shared context diagnostic                              | `compound parts require a matching root provider`                         |
| C9  | SSR and hydration          | isolated render/mount   | `defaultActiveId` renders on links and items and hydrates without warnings                  | `toc-ssr.test.ts`                                                         |
