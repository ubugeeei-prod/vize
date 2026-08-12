# Focus scope behavior contract

`createFocusScope` provides the focus lifecycle used by dialogs, popovers, menus, and composite
widgets without owning roles, rendering, or styling. The consumer decides which UI is modal and
combines this primitive with inerting and scroll locking when the interaction requires them.

| Concern           | Contract                                                                                                                                                        |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Entry             | Optional automatic focus resolves an explicit target, the first programmatically focusable descendant, or an explicit root fallback.                            |
| Containment       | Only the latest containing scope owns document-level recovery and Tab wrapping; non-containing children remain valid portal destinations.                       |
| Nesting           | Scope order is stable across same-document root replacement, and a nested containing scope temporarily supersedes its ancestors.                                |
| Restoration       | The entry target is restored when usable; removed targets fall forward, then backward, then to the parent scope without stealing deliberate outside focus.      |
| Traversal         | Positive tabindex order, native controls, programmatic targets, radio groups, disabled fieldsets, collapsed details, inert and hidden ancestors are normalized. |
| Shadow DOM        | Open shadow roots and assigned slots follow composed-tree order; unassigned light content and independent shadow radio groups stay isolated.                    |
| Portals           | A containing parent owns later non-containing scope roots even when they are not DOM descendants.                                                               |
| Reactivity        | Root, containment, entry, restoration, and movement filters are read at their documented operation boundaries.                                                  |
| Failure atomicity | Entry and restoration failures aggregate after listeners, observers, stack ownership, and reactive state are consistently cleaned up.                           |
| Server rendering  | Setup performs no global DOM access; activation waits for component mount and a nullable root can attach during hydration.                                      |
| Vapor             | A public composable fixture must compile without diagnostics in native DOM, SSR, and Vapor lanes.                                                               |
| Styling           | The module emits no CSS and exposes no style assumptions.                                                                                                       |
| Tree shaking      | Root and subpath consumers emit identical JavaScript, retain no unrelated families, and emit zero CSS.                                                          |
