# Spatial navigation behavior contract

`createSpatialNavigation` turns collection geometry into predictable physical arrow movement. It
implements library-owned behavior because CSS Spatial Navigation remains a Working Draft and is not
a portable browser primitive. Roles, selection, activation, and styling remain consumer-owned.

| Concern           | Contract                                                                                                                                                |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Geometry          | Default measurement uses transformed viewport rectangles from `getBoundingClientRect`; custom geometry supports virtualized and server-described items. |
| Normal ranking    | Candidates use the CSS Spatial Navigation distance terms: Euclidean distance, orthogonal displacement and bias, and projected alignment.                |
| Grid ranking      | Aligned candidates are preferred, then primary-axis distance, overlap, orthogonal distance, and registry order.                                         |
| Direction         | Up, down, left, and right are physical directions and do not reverse under RTL.                                                                         |
| Candidate set     | Only registry-navigable items with finite, non-negative, ordered geometry participate.                                                                  |
| Initial state     | A missing active key uses the first navigable item as the search origin without mutating it during `findTarget`.                                        |
| Boundaries        | `contain` consumes an owned boundary arrow; `exit` publishes the boundary but preserves native scroll or ancestor behavior.                             |
| Looping           | Optional looping selects the opposite aligned spatial edge and remains deterministic by registry order.                                                 |
| Editing           | Modified, composing, already-handled, form-control, contenteditable, and shadow-retargeted editor events are preserved.                                 |
| Focus             | `focus` moves DOM focus; `logical` only changes active state for active-descendant or externally managed focus.                                         |
| Scrolling         | `preventScroll` is forwarded with a legacy focus fallback; logical focus and prevented focus use nearest-block reveal by default.                       |
| Virtualization    | Null DOM elements are valid with custom rectangles and logical focus; custom reveal receives the resolved collection item.                              |
| Reactivity        | Algorithm, boundaries, looping, focus, prevent-scroll, and disablement are read at operation time.                                                      |
| Callback timing   | State and DOM effects commit before immutable `onNavigate`; the originating native event and numeric score are retained.                                |
| Failure atomicity | Registry, focus, reveal, and consumer failures are aggregated after committed logical state remains observable.                                         |
| Server rendering  | Construction and prop reads do not measure layout; deterministic markup hydrates in place before first navigation.                                      |
| Vapor             | A public composable fixture must compile without diagnostics in native DOM, SSR, and Vapor lanes.                                                       |
| Styling           | The module emits no CSS and owns no visual state.                                                                                                       |
| Tree shaking      | Root and subpath consumers emit identical JavaScript, retain no unrelated families, and emit zero CSS.                                                  |
