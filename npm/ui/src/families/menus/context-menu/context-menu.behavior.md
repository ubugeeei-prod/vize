# ContextMenu behavior contract

`@vizejs/ui/context-menu` opens the shared menu core (see `../menu/menu.behavior.md`) anchored
at a viewport point. `context-menu-root.vue` owns a virtual anchor and restores focus to
whatever held it before the request; `context-menu-trigger.vue` is a non-interactive region that
translates `contextmenu`, Shift+F10, the ContextMenu key, and touch long-press into requests.
Every row is proven by the named test in `context-menu.test.ts` or `context-menu-ssr.test.ts`.

| #   | State         | Input                           | Outcome                                                                                                           | Proven by                                                                                   |
| --- | ------------- | ------------------------------- | ----------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| C1  | closed        | `contextmenu` (right click)     | prevents the native menu, anchors at the pointer, opens `data-menu-kind="context-menu"` content, focuses the menu | `contextmenu opens at the pointer, prevents the native menu, and focuses the menu`          |
| C2  | open          | new request elsewhere           | the outside press closes, the request reopens the same content at the new anchor                                  | `a new request while open re-anchors the same open menu`                                    |
| C3  | closed, focus | Shift+F10 / ContextMenu key     | anchors at the focused element, focuses the first item; Escape returns focus to the requesting element            | `Shift+F10 and the ContextMenu key open at the focused element with the first item focused` |
| C4  | closed        | plain F10 / other keys          | nothing opens                                                                                                     | `plain F10 and other keys do not open the menu`                                             |
| C5  | closed, touch | press held for `longPressDelay` | opens at the touch point; shorter taps do nothing                                                                 | `a touch long press opens at the touch point; short taps do not`                            |
| C6  | closed, mouse | press held                      | mouse presses never use the long-press path                                                                       | `mouse presses never trigger the long-press path`                                           |
| C7  | disabled      | `contextmenu`                   | trigger or root `disabled` lets the native menu show and publishes `data-disabled`                                | `disabled triggers and roots let the native menu show`                                      |
| C8  | enabled       | prevented `contextmenu` emit    | the native menu is kept and nothing opens                                                                         | `a prevented contextmenu emit keeps the native menu`                                        |
| C9  | open          | select / outside pointer-down   | selection closes; outside pointer-down closes                                                                     | `selecting closes; outside pointer-down closes without restoring focus`                     |
| C10 | any           | `openAt(point)` expose          | opens programmatically with the first item focused                                                                | `exposes openAt for programmatic requests`                                                  |
| C11 | SSR           | two requests + hydrate          | byte-identical markup, no long-press description leakage, warning-free hydration                                  | `renders identical context-menu markup across SSR requests and hydrates cleanly`            |
