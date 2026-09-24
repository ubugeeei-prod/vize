# DropdownMenu behavior contract

`@vizejs/ui/dropdown-menu` is the WAI-ARIA APG menu button built on the shared menu core
(see `../menu/menu.behavior.md`). `dropdown-menu-root.vue` publishes `data-menu-kind="dropdown-menu"`;
`dropdown-menu-trigger.vue` adds press-to-open pointer semantics. Items, groups, submenus,
and content are the Menu SFCs re-exported under `DropdownMenu*` names. Every row is proven by
the named test in `dropdown-menu.test.ts` or `dropdown-menu-ssr.test.ts`.

| #   | State            | Input                                        | Outcome                                                                                                       | Proven by                                                                     |
| --- | ---------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| D1  | closed / open    | primary mouse pointer-down                   | toggles; opening prevents trigger focus and focuses the content; the same press's click does not toggle again | `mouse pointer-down toggles the menu and focuses the content`                 |
| D2  | closed           | secondary button, ctrl-press, touch tap      | secondary and ctrl presses are ignored; touch taps toggle on click                                            | `secondary buttons and ctrl-clicks do not toggle; touch taps toggle on click` |
| D3  | `openOn="click"` | pointer-down then click                      | only the click toggles                                                                                        | `openOn click waits for the click`                                            |
| D4  | closed           | ArrowUp / keyboard or AT click (`detail: 0`) | opens with the last or first item focused; Escape returns focus to the trigger                                | `keyboard and assistive-technology activation focus the first or last item`   |
| D5  | closed           | prevented `pointerdown` / `keydown` emits    | state is unchanged                                                                                            | `preventable pointerdown and keydown emits keep the menu closed`              |
| D6  | disabled trigger | any activation                               | native `disabled`; nothing opens                                                                              | `disabled triggers ignore every activation`                                   |
| D7  | open             | keyboard into a submenu and select           | shared menu parts: submenu keyboard entry, selection closes every level and returns focus to the trigger      | `items and submenus are the shared menu parts`                                |
| D8  | SSR              | two requests + hydrate                       | byte-identical markup with `data-menu-kind="dropdown-menu"` and warning-free hydration                        | `renders identical dropdown markup across SSR requests and hydrates cleanly`  |
