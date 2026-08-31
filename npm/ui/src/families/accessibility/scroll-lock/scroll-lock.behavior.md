# Scroll lock behavior contract

`createScrollLock` freezes one document's layout viewport while leaving focus, outside inerting,
dismissal, visual styling, and overlay semantics to independently composable primitives.

| Concern          | Contract                                                                                                                                            |
| ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Ownership        | Document locks are reference-counted; the document restores only after its final enabled owner exits.                                               |
| Strategy         | `overflow` locks the root scroll container; `fixed` also fixes the body; `auto` selects fixed positioning only for iOS-like touch platforms.        |
| Composition      | A nested fixed owner upgrades the whole stack and releasing it returns atomically to overflow locking without exposing an unlocked frame.           |
| Restoration      | Owned inline values, priorities, the data attribute, and the captured layout-viewport offset restore exactly.                                       |
| Scrollbar gap    | Root `scrollbar-gutter: stable` preserves classic gutters where supported; a measured logical-padding fallback and CSS custom property are exposed. |
| Direction        | Gap fallback uses `padding-inline-end`; native gutters remain user-agent positioned for platform and bidirectional conventions.                     |
| Zoom             | No `touch-action` or gesture cancellation is installed, so page zoom and visual-viewport panning remain browser controlled.                         |
| Viewports        | Scroll capture uses layout-viewport `scrollX`/`scrollY`; pinch zoom and virtual keyboards may independently move or resize the visual viewport.     |
| Documents        | Reactive migration restores the old document before acquiring the new document; iframe documents keep independent stacks.                           |
| Server rendering | Setup performs no global DOM access; a nullable document joins the stack only after hydration mount.                                                |
| Vapor            | A public composable fixture must compile without diagnostics in native DOM, SSR, and Vapor lanes.                                                   |
| Styling          | The package emits no CSS; `[data-vize-scroll-locked]` and `--vize-scroll-lock-scrollbar-gap` are explicit user styling hooks.                       |
| Tree shaking     | Root and subpath consumers emit identical JavaScript, retain no unrelated families, and emit zero CSS.                                              |

The root-element gutter behavior follows CSS Overflow Level 3. Layout and visual viewport
coordinates follow CSSOM View. The primitive deliberately does not suppress Pointer Events
`touch-action` behaviors because disabling pan and pinch gestures would also disable user zoom.
