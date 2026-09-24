# Window manager behavior contract

Normative state x input -> outcome table for `window-manager.vue`,
`floating-window.vue`, and `window-dock.vue` (`@vizejs/ui/window-manager`). Rows
are proven by `window-manager-model.test.ts`, `window-manager.test.ts`, and
`window-manager-ssr.test.ts`; compile-only assertions live in
`window-manager.types.test-d.ts`.

| #    | State                  | Input                               | Outcome                                                                                                  | Proven by                                                                                                                  |
| ---- | ---------------------- | ----------------------------------- | -------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| WM1  | known / unknown bounds | clamp                               | rects stay inside known bounds (pinned top-left when too large); unknown bounds leave them untouched     | `clamps rects inside known bounds and leaves unknown bounds untouched`                                                     |
| WM2  | moving rect            | snap                                | edges within the threshold snap to bounds and neighbour edges; `0` disables snapping                     | `snaps edges to bounds and neighbours within the threshold`                                                                |
| WM3  | rect, delta            | move                                | the rect moves, snaps, then clamps                                                                       | `moves by a delta, then snaps and clamps`                                                                                  |
| WM4  | rect, edge, delta      | resize                              | the opposite edge stays fixed and size respects min/max constraints and bounds                           | `resizes from every edge within constraints and bounds`                                                                    |
| WM5  | stacking order         | raise                               | the window moves to the front (unknown ids append; front stays the same array)                           | `raises windows to the front of the stacking order`                                                                        |
| WM6  | stored text            | parse                               | valid windows and known order ids survive; malformed input yields `undefined`                            | `parses stored layouts and drops malformed entries`                                                                        |
| WM7  | defaults               | render                              | labelled non-modal `dialog`s use default geometry, later windows stack higher, 8 resize handles render   | `renders labelled non-modal windows with default geometry and stacking`                                                    |
| WM8  | normal window          | pointer down, drag handle           | the window becomes active and frontmost, moves with the pointer, and stops after pointerup               | `pointer focus raises a window and dragging the handle moves it`                                                           |
| WM9  | normal window          | resize handle, arrows, Shift+arrows | resize respects minimums; arrows move and Shift+arrows resize by `keyboardStep`                          | `resize handles and keyboard move/resize respect constraints`                                                              |
| WM10 | any window             | minimize/maximize/dock/close        | minimized windows are `hidden`, maximized fill the manager, dock buttons restore/minimize, close emits   | `minimize, maximize, restore, close, and dock activation`                                                                  |
| WM11 | controlled, measured   | drag near an edge                   | controlled layouts render as given; moves emit snapped, clamped layouts                                  | `controlled layouts drive geometry and snapping clamps to measured bounds`                                                 |
| WM12 | storage                | mount, change                       | stored layouts load after mount, changes are written back, malformed data and missing scopes are handled | `persists layouts after mount and ignores malformed storage`                                                               |
| WM13 | no provider            | mount window or dock                | stable `VIZE_UI_CONTEXT_MISSING: WindowManager` diagnostic                                               | `windows and docks outside a manager throw the context diagnostic`                                                         |
| WM14 | SSR                    | isolated requests, hydrate          | byte-identical geometry/stacking/dock markup; hydration without warnings                                 | `renders byte-identical window geometry, stacking, and dock markup`, `hydrates windows and dock without mismatch warnings` |
| WM15 | DOM/SSR/Vapor          | compile                             | every SFC compiles in every renderer lane                                                                | `scripts/check-renderers.ts`                                                                                               |

Windows are non-modal `role="dialog"` regions named by `title`; the drag
handle is a focusable element spread by the consumer (`handleProps`) that
advertises its arrow-key shortcuts.
