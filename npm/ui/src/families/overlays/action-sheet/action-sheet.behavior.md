# ActionSheet behavior contract

Normative state x input -> outcome table for `action-sheet-menu.vue` and
`action-sheet-item.vue` (`@vizejs/ui/action-sheet`). The sheet container is the
Drawer (bottom side by default: native modal `<dialog>`, focus scope, drag to
dismiss, snap points), re-exported as `ActionSheet`, `ActionSheetTrigger`,
`ActionSheetContent`, `ActionSheetTitle`, `ActionSheetDescription`, and
`ActionSheetCancel`; the actions follow the WAI-ARIA APG menu pattern. Every row
is proven by the named test in `action-sheet.test.ts` or `action-sheet-ssr.test.ts`.

| #   | State           | Input                             | Outcome                                                                                                                                              | Proven by                                                                              |
| --- | --------------- | --------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| AS1 | closed          | trigger click                     | a bottom Drawer opens; the `role="menu"` is labelled by the sheet title; the first enabled action is the tab stop; disabled/destructive hooks render | `opens a bottom sheet whose actions form a menu labelled by the title`                 |
| AS2 | open            | Arrow / Home / End / typeahead    | roving focus moves between enabled actions, wraps with `loop`, and typeahead matches labels                                                          | `arrow keys, Home, End, and typeahead move a roving focus that skips disabled actions` |
| AS3 | open            | click / Enter / Space             | `select` fires with a cancelable event; unless prevented the sheet closes; disabled actions never select                                             | `selecting an action emits select and closes; preventDefault keeps the sheet open`     |
| AS4 | no provider     | setup                             | the menu needs the Dialog context and items need the menu (`VIZE_UI_CONTEXT_MISSING`)                                                                | `menus and items require their providers`                                              |
| AS5 | SSR / hydration | isolated requests (`defaultOpen`) | byte-identical menu markup with a deterministic tab stop and hydration without diagnostics                                                           | `renders an open sheet's menu deterministically and hydrates without mismatches`       |

The subpath ships no CSS.
