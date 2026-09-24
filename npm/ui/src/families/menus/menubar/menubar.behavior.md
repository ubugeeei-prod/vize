# Menubar behavior contract

`@vizejs/ui/menubar` implements the WAI-ARIA APG menubar pattern on the shared menu core
(see `../menu/menu.behavior.md`). `menubar-root.vue` owns the open-menu value and a roving
tab stop across triggers; each `menubar-menu.vue` is a non-modal menu root; `menubar-trigger.vue`
is a `role="menuitem"` button. Every row is proven by the named test in `menubar.test.ts` or
`menubar-ssr.test.ts`.

| #   | State              | Input                                  | Outcome                                                                                                            | Proven by                                                                      |
| --- | ------------------ | -------------------------------------- | ------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------ |
| B1  | any                | render                                 | `role="menubar"`, horizontal orientation, label, `menuitem` triggers with `aria-haspopup`; one `tabindex="0"` stop | `renders a labelled horizontal menubar with a single roving tab stop`          |
| B2  | trigger focus      | ArrowLeft/Right, Home/End, characters  | roving focus moves (wrapping by default), jumps to ends, and typeahead matches trigger text                        | `Left/Right move the roving focus with wrapping, Home/End jump`                |
| B3  | `dir="rtl"`        | ArrowLeft                              | moves to the next trigger                                                                                          | `rtl flips Left/Right across triggers`                                         |
| B4  | trigger focus      | ArrowDown/Enter/Space, ArrowUp, Escape | opens with the first or last item focused and updates `v-model`; Escape closes back to the trigger                 | `ArrowDown/Enter/Space open with the first item; ArrowUp with the last`        |
| B5  | menu open          | ArrowRight/Left on a plain item        | closes the menu and opens the adjacent one with its first item focused, wrapping                                   | `Right/Left inside an open menu hand off to the adjacent menu`                 |
| B6  | submenu in a menu  | ArrowRight / ArrowRight in submenu     | a submenu trigger consumes ArrowRight; ArrowRight on a plain submenu item hands off to the next menu               | `a submenu trigger consumes Right; Left closes the submenu before handing off` |
| B7  | menu open          | ArrowRight on a focused trigger        | the next menu opens                                                                                                | `Right on a trigger while a menu is open opens the next menu`                  |
| B8  | mouse              | pointer-down / hover siblings          | pointer-down toggles; while a menu is open, hovering another trigger switches the open menu                        | `pointer-down toggles a menu and hovering siblings switches the open menu`     |
| B9  | menu open          | render / outside pointer-down          | menubar menus are non-modal; outside pointer-down closes                                                           | `menubar menus are non-modal and close on outside pointer-down`                |
| B10 | disabled menu      | navigation / activation                | its trigger remains focusable with `aria-disabled` and never opens                                                 | `disabled menus keep a focusable trigger that never opens`                     |
| B11 | controlled `value` | selection / props / expose             | the prop wins until accepted; `setValue` and `focus` drive the root                                                | `controlled value and the exposed API drive the open menu`                     |
| B12 | SSR                | two requests + hydrate                 | byte-identical markup with a single tab stop and warning-free hydration                                            | `renders identical menubar markup across SSR requests and hydrates cleanly`    |
