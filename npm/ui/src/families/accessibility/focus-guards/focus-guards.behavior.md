# Focus guards behavior contract

`createFocusGuards` supplies consumer-rendered before and after sentinels for a primary focus
region plus optional portalled branches. It complements focus containment; it does not assign
dialog semantics, inert outside content, or lock scrolling.

| Concern          | Contract                                                                                                                      |
| ---------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| Direction        | A before sentinel enters at the first target and wraps backward to the last; an after sentinel performs the inverse.          |
| Evidence         | Tab direction and `relatedTarget` distinguish entry from wrapping; redirects publish immutable, preventable events.           |
| Ordering         | Positive tabindex values precede normal sequential order across the root, portals, open shadows, and rendered slots.          |
| Nesting          | Only the latest enabled, connected owner in a document exposes `tabindex="0"`; releasing it synchronously resumes its parent. |
| Recovery         | Document mutations detect disconnected and reinserted roots; `refresh()` also reconciles imperative changes.                  |
| Fallback         | An owned explicit fallback or a focusable root handles regions without a remaining sequential target.                         |
| Accessibility    | Sentinels omit `aria-hidden`, which must not be placed on focusable content; preventing redirect requires visible focus.      |
| Styling          | No CSS is emitted. A frozen invisible inline-style preset and data attributes are optional consumer hooks.                    |
| Server rendering | Nullable roots render deterministic inactive sentinels and activate only after hydration mount.                               |
| Vapor            | A public composable fixture must compile without diagnostics in native DOM, SSR, and Vapor lanes.                             |
| Tree shaking     | Root and subpath consumers emit identical JavaScript, retain no unrelated families, and emit zero CSS.                        |

Place the sentinels immediately before and after the rendered region in sequential DOM order. Portalled
branches participate in destination discovery even though their DOM location is independent.

The redirect contract follows the HTML sequential focus navigation model and the WAI-ARIA modal dialog
requirement that Tab and Shift+Tab remain within the active dialog. Focus order must remain meaningful
under WCAG 2.2 Success Criterion 2.4.3.
