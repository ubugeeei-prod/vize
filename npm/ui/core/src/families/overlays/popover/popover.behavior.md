# Popover behavior contract

| Surface   | Contract                                                                                                                                                                                    |
| --------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| State     | `PopoverRoot` supports uncontrolled `defaultOpen` and controlled `open` with `update:open` and `open-change` events; disabled roots request closure and publish `data-disabled`.            |
| Trigger   | `PopoverTrigger` renders a native button with deterministic ids, `aria-haspopup="dialog"`, `aria-expanded`, `aria-controls`, `part="trigger"`, `data-state`, and disabled data hooks.       |
| Content   | `PopoverContent` composes `Portal`, `Presence`, `Positioner`, `DismissableLayer`, and optional focus/isolation controllers; it renders `role="dialog"` with deterministic content ids.      |
| Dismissal | Escape, outside pointer-down, and outside focus request closing unless the preventable callback was canceled; trigger interaction is treated as an inside branch and toggles through state. |
| Focus     | Open content auto-focuses the provided `initialFocus`, the first eligible descendant, or the content fallback; close restores focus to the trigger when focus restoration is enabled.       |
| Modal     | `modal` content may contain focus, add focus guards, inert outside content, and lock scroll; non-modal content skips those document-isolating effects while preserving dialog semantics.    |
| Portal    | Content teleports to `to` after hydration by default; `portalDisabled` renders in place and `forceMount` keeps closed content hidden without activating document controllers.               |
| Position  | `placement`, `direction`, collision, arrow, safe-area, and size props forward to `Positioner`; content publishes `data-placement`, `data-side`, `data-align`, and `data-top-layer`.         |
| Arrow     | `PopoverArrow` is a measured `Positioner` arrow with `part="arrow"`, Popover state data hooks, and slot coordinates; it is decorative and emits no role or label of its own.                |
| Styling   | Popover emits no component CSS beyond scoped empty blocks; consumers style with `data-vize-ui`, `part`, `data-state`, `data-side`, `data-align`, and Positioner CSS variables.              |
| CSS vars  | `PopoverContent` forwards Positioner sizing variables, including `--vize-ui-positioner-available-width` and `--vize-ui-positioner-available-height`, when `size` is enabled.                |
| SSR       | Server output is deterministic, renders portal content in place, includes no document listeners, and activates dismissal, focus, inert, and scroll-lock controllers only after mount.       |

| Component             | State x input                           | Outcome                                                                                                             |
| --------------------- | --------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `popover-root.vue`    | controlled or uncontrolled open request | Publishes `open`, `modal`, `disabled`, ids, and `data-state`; emits state requests only for distinct open changes.  |
| `popover-trigger.vue` | enabled click                           | Emits `click`, then toggles the root when the event was not canceled.                                               |
| `popover-content.vue` | open                                    | Renders positioned dialog content, activates dismissable layer, applies focus lifecycle, and exposes focus helpers. |
| `popover-content.vue` | open modal                              | Adds focus guards, inert outside, focus containment, and scroll lock through shared foundation primitives.          |
| `popover-content.vue` | closed and force-mounted                | Keeps content in the DOM with `hidden` state while document controllers remain inactive.                            |
| `popover-arrow.vue`   | inside positioned content               | Registers with `Positioner`, publishes arrow geometry to CSS, and exposes slot coordinates.                         |

| Event                  | Dispatch timing and payload                                                               |
| ---------------------- | ----------------------------------------------------------------------------------------- |
| `open-auto-focus`      | Before automatic entry focus; payload is a preventable focus-scope event.                 |
| `close-auto-focus`     | Before automatic focus restoration; payload is a preventable focus-scope event.           |
| `escape-key-down`      | Before Escape dismissal; payload is the preventable dismissable-layer Escape event.       |
| `pointer-down-outside` | Before pointer dismissal; payload is the preventable outside pointer event.               |
| `focus-outside`        | Before focus dismissal; payload is the preventable outside focus event.                   |
| `interact-outside`     | Before pointer or focus dismissal; payload is the preventable outside interaction event.  |
| `dismiss`              | After an unprevented dismissal request; payload records reason, target, and native event. |
