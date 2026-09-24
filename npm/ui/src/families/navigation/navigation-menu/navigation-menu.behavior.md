# Navigation Menu Behavior Contract

Normative state x input -> outcome table for the `@vizejs/ui/navigation-menu`
parts: `navigation-menu-root.vue`, `navigation-menu-list.vue`,
`navigation-menu-item.vue`, `navigation-menu-trigger.vue`,
`navigation-menu-content.vue`, `navigation-menu-link.vue`,
`navigation-menu-indicator.vue`, and `navigation-menu-viewport.vue`. Every row
is proven by the named test in `navigation-menu.test.ts` or
`navigation-menu-ssr.test.ts`; compile-only guarantees live in
`navigation-menu.types.test-d.ts`.

The menu follows the APG disclosure navigation pattern (not `role="menu"`): a
labelled `<nav>` with a `<ul>` of items, native `<button aria-expanded
aria-controls>` triggers, and flyouts rendered inline after their trigger so
the native tab order reaches them. The viewport is a measured backdrop that
publishes `--vize-navigation-menu-viewport-width/height`; the indicator
publishes `--vize-navigation-menu-indicator-offset/size`. Both are measured on
the client only.

| ID  | State                     | Input                                    | Outcome                                                                                           | Evidence                                                                                   |
| --- | ------------------------- | ---------------------------------------- | ------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| N1  | closed                    | render                                   | labelled `nav`, deterministic trigger/content ids, closed flyouts unmounted, `aria-current` links | `renders a labelled disclosure navigation with wired triggers and closed flyouts`          |
| N2  | any                       | trigger click                            | toggles the flyout inline after the trigger and reports `toggle`                                  | `click toggles a flyout inline after its trigger`                                          |
| N3  | mouse or pen              | hover, move, leave, re-enter             | opens after `delayDuration`, switches instantly, closes after `closeDelay`, skip window reopens   | `hover opens after the delay, skips it between triggers, and closes after a grace period`  |
| N4  | touch                     | pointerenter                             | hover intent is ignored; touch uses click                                                         | `touch pointers never open on hover`                                                       |
| N5  | focus on a list entry     | arrows, Home, End, open key              | roving between triggers and top-level links (wrapping); ArrowDown opens and focuses the flyout    | `arrow keys move between top-level entries and the open key enters the flyout`             |
| N6  | open                      | Escape, outside pointer, focus out, link | dismisses; Escape restores focus to the trigger; flyout links close with reason `link`            | `Escape, outside pointerdown, focus leaving, and link selection dismiss the flyout`        |
| N7  | switching items           | open another item, `forceMount`          | `data-motion` reports from/to start/end; force-mounted flyouts stay `hidden` while closed         | `motion attributes follow the direction between items and forceMount keeps hidden flyouts` |
| N8  | controlled / exposed root | click, setProps, open/close              | the parent owns the open value; imperative open/close report distinct changes                     | `controlled values wait for the parent and the root exposes open/close`                    |
| N9  | open                      | measure                                  | indicator and viewport publish CSS variables for the open item and clear them when closed         | `the indicator and viewport publish geometry for the open item`                            |
| N10 | disabled trigger          | click, hover, open key                   | the flyout never opens                                                                            | `disabled triggers ignore click, hover, and the open key`                                  |
| N11 | missing provider          | setup                                    | parts fail closed with the shared context diagnostic                                              | `compound parts require matching providers`                                                |
| N12 | SSR and hydration         | isolated render/mount                    | default-open flyouts render, geometry is omitted, hydration has no warnings                       | `navigation-menu-ssr.test.ts`                                                              |
| N13 | link `href` / `as`        | render                                   | script-capable hrefs are dropped; `as` renders router or custom elements                          | `links drop script-capable hrefs and render custom elements`                               |
