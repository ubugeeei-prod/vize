# SplitButton behavior contract

`@vizejs/ui/split-button` pairs a primary action with a WAI-ARIA APG menu button inside one
named `role="group"`. It is built on the shared menu core (see `../../menus/menu/menu.behavior.md`):
`split-button-root.vue` publishes `data-menu-kind="split-button"`, `split-button-action.vue` is the
primary native button, and `split-button-trigger.vue` is the menu button. Items, groups, submenus,
and content are the Menu SFCs re-exported under `SplitButton*` names. Every row is proven by the
named test in `split-button.test.ts` or `split-button-ssr.test.ts`.

| #   | State                 | Input                                           | Outcome                                                                                                                                                               | Proven by                                                                                      |
| --- | --------------------- | ----------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| S1  | closed                | render; press the primary action                | a named `role="group"` holds the action then the trigger; only the trigger carries `aria-haspopup="menu"`; pressing the action emits `press` and never opens the menu | `renders a named group with a primary action and a menu button`                                |
| S2  | closed / open         | primary mouse pointer-down on the trigger       | toggles; opening prevents trigger focus and focuses the content; `aria-expanded`, `aria-controls`, and every part's `data-state` follow                               | `mouse pointer-down on the trigger toggles the menu and focuses the content`                   |
| S3  | closed                | secondary button, ctrl-press, touch tap, click  | secondary and ctrl presses are ignored; touch taps and `openOn="click"` toggle on click                                                                               | `secondary buttons and ctrl-clicks do not toggle; touch taps and openOn click toggle on click` |
| S4  | closed                | Enter / Space / ArrowDown / ArrowUp on trigger  | opens with the first or last item focused; Escape closes and returns focus to the trigger                                                                             | `trigger keyboard opens on the first or last item and Escape returns focus to it`              |
| S5  | closed                | Alt+ArrowDown on the primary action             | opens with the first item focused; closing returns focus to the action; plain ArrowDown is not consumed                                                               | `Alt+ArrowDown on the primary action opens the menu and focus returns to the action`           |
| S6  | closed                | prevented `keydown` / `pointerdown` emits       | state is unchanged                                                                                                                                                    | `preventable keydown and pointerdown emits keep the menu closed`                               |
| S7  | open                  | select an item, including from a submenu        | every level closes, focus returns to the trigger, and the primary `press` does not fire                                                                               | `selecting an item closes every level and returns focus to the trigger`                        |
| S8  | open → disabled root  | `disabled` becomes true                         | the menu closes; both buttons are natively disabled; the group publishes `aria-disabled` and `data-disabled`                                                          | `a disabled root disables both buttons and closes an open menu`                                |
| S9  | part disabled         | `disabled` on the action or on the trigger only | the other part keeps working                                                                                                                                          | `the action and the trigger can be disabled independently`                                     |
| S10 | open                  | `anchor="group"` (default) / `anchor="trigger"` | the menu is positioned against the whole group or against the trigger                                                                                                 | `the menu is positioned against the group by default and the trigger on request`               |
| S11 | SSR                   | two requests + hydrate                          | byte-identical markup with `data-menu-kind="split-button"` and warning-free hydration                                                                                 | `renders identical split button markup across SSR requests and hydrates cleanly`               |
| S12 | SSR closed / disabled | render                                          | no menu, no `aria-controls`; a disabled root renders two disabled buttons                                                                                             | `a closed or disabled split button renders no menu on the server`                              |

## Public surface

| Kind   | Surface                                                                                                                                             |
| ------ | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| parts  | `root` (group), `action` (primary button), `trigger` (menu button); menu parts are documented in the Menu contract                                  |
| data   | `data-vize-ui` (`split-button`, `split-button-action`, `split-button-trigger`), `data-state` (`open` / `closed`), `data-disabled`                   |
| emits  | root: `update:open`, `openChange`; action: `press`, `keydown`; trigger: `pointerdown`, `keydown`                                                    |
| expose | root: the Menu root surface plus `disabled` and `element`; action and trigger: `element`, `focus()`                                                 |
| naming | the group needs `aria-label` or `aria-labelledby`; an icon-only trigger needs `aria-label` because it also names the menu through `aria-labelledby` |
