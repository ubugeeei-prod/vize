# SwipeActions behavior contract

Normative state x input -> outcome table for `swipe-actions.vue`,
`swipe-actions-content.vue`, `swipe-actions-tray.vue`, and
`swipe-actions-action.vue` (`@vizejs/ui/swipe-actions`). Every row is proven by
the named test in `swipe-actions.test.ts` or `swipe-actions-ssr.test.ts`.

| #   | State                       | Input                                 | Outcome                                                                                                                                                                           | Proven by                                                                         |
| --- | --------------------------- | ------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| SW1 | closed                      | render                                | focusable content with an arrow-key hint (`aria-describedby`, `aria-keyshortcuts`), `inert` trays (`role="group"`), `--vize-swipe-offset: 0px`                                    | `renders a focusable row with inert trays and a keyboard hint`                    |
| SW2 | any                         | horizontal drag                       | past a 6px slop the content follows the pointer (`data-state="dragging"`); release opens a tray past half its width, else closes; open trays rest at their width and lose `inert` | `dragging reveals trays, snaps open past half the tray, and snaps back otherwise` |
| SW3 | any                         | vertical drag / missing tray / cancel | vertical gestures scroll natively, sides without trays never move, and `pointercancel` restores the previous position                                                             | `vertical drags and cancels never open, and sides without trays do not move`      |
| SW4 | `fullSwipe`                 | long drag                             | past `fullSwipeThreshold` of the row width `data-full-swipe` names the side and release emits `fullSwipe(side)` then closes; disabled full swipe just opens                       | `a long swipe past the threshold emits fullSwipe and closes`                      |
| SW5 | focused content             | Arrow / Escape / action               | arrows reveal the tray toward which content moves and focus its first action; Escape or running an action closes and returns focus                                                | `arrow keys reveal trays and focus the first action; Escape and actions close`    |
| SW6 | RTL / controlled / disabled | keys, API, drags                      | RTL mirrors keys and offsets; controlled `open` waits for the parent; disabled rows leave the tab order and ignore input                                                          | `RTL mirrors keys and drag direction; controlled and disabled rows`               |
| SW7 | no provider                 | setup                                 | parts throw `VIZE_UI_CONTEXT_MISSING: SwipeActions`                                                                                                                               | `parts require a SwipeActions provider`                                           |
| SW8 | SSR / hydration             | isolated requests                     | byte-identical closed markup and hydration without diagnostics                                                                                                                    | `renders byte-identical closed rows and hydrates without mismatches`              |

The subpath ships no CSS.
