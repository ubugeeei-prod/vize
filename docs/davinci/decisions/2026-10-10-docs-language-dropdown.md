# Authored documentation language dropdown

The language control needs more room beside its chevron and readable, authored
options. This is a low-priority documentation UI change, prepared independently
of the navigation/readability delivery in #8384. The scoped tracking issue is
[#8492](https://github.com/ubugeeei-prod/vize/issues/8492), with no P0/P1 priority.
The [paired issue decision](https://github.com/ubugeeei-prod/vize/issues/8492#issuecomment-6096468088)
records the same scope and pending delivery.

## Interaction and ownership

- Keep all five existing locale names and the current locale-to-page mapping.
  Preserve the current search string and hash when changing language, including
  anchor/history changes after installation. Refresh native hrefs when opening
  and before pointer/context-menu or Enter activation so modified clicks work.
- Use a disclosure button and ordinary language links. Links keep native
  navigation, Enter, modified clicks, and browser Tab order. This is navigation,
  so the options do not claim listbox or application-menu semantics.
- Expose the button's expanded state and controlled region. Mark the current
  locale with `aria-current="page"`, a visible check, and a background/edge marker.
- Arrow keys, Home and End move focus among links. Escape closes this disclosure,
  restores button focus, and consumes the event before the mobile sidebar sees
  it. Outside clicks and focus moving outside dismiss without stealing focus.
- Repeated initialization updates the existing control. Replacement of the
  header disposes the old control and its event listeners; detached controls
  are observed and cleaned up. SPA selection closes the popup and updates the
  current language in the same mount. No listener is attached to each locale link.
- Keep the existing theme palette and header/sidebar layout. Give the trigger
  and options 44px minimum height, a 12px text-to-chevron gap, and 14.4px right
  padding. Preserve the compact mobile label and fit the option panel to the
  viewport. The existing 768px breakpoint hides other header actions. Move only
  this same language mount beside the brand on mobile and back into the actions
  on desktop, preserving its open state and owned focus during resize.

The implementation lives in `docs/theme/i18n/locale-switcher.js` and the existing
`locale-selector.css`. The existing navigation module supplies its unchanged
page-path function. `background.ts` loads the new module before navigation;
the original complete script-order contract includes that wiring.

## Delivery and acceptance

The original preparation used actual main
`b3f86a13d51fad25b7a36aa89e2d42a2c13859ea`. The delivery child starts from the
exact #8389 parent `256dfc20d6e3664e48fb348f8300061e27d6565c`, whose #8384
parent is `2ba3daca0878b59986e49d5351f894b677c6217d`. Neither parent head is
rewritten. The language installer/CSS remains independent of sidebar, code-color
and capture-control repairs. Those two parents actually merged as `db0ec3dd`
and `538c07ac`; the remaining #8493 child is composed once on actual main
`538c07ac7eca17db3151fe68cdf38586bc561bb0` together with its bounded receipt
review repair. Preserve both historical parent heads in native Stack #8411,
retarget the open child to main, and require fresh exact-head qualification.
Every incoming canonical decision byte survives the sole owned clause addition.

`docs/scripts/locale-render-assertions.ts` retains the complete five-link packet,
native modified-click destination, keyboard/dismissal states and actual
repeat/replacement/detachment receipts. The actual `verify-navigation-render.ts`
caller now invokes `locale-control-render.ts` on separate real pages so lifecycle
replacement cannot disturb the original navigation/theme/geometry assertions.
All existing routes, geometry budgets and assertions remain unchanged.
The companion `locale-responsive-assertions.ts` crosses the actual breakpoint
in both directions and checks 320/360/390px bounds, focus and mount identity.

Required browser acceptance uses the actual docs theme on English/Japanese pages,
desktop/mobile and light/dark: selected/hover/focus appearance, measured chevron
spacing, all five hrefs including query/hash, keyboard navigation/dismissal,
outside dismissal, and repeat/replacement initialization without duplicates.
The normal campaign retains all eight visual-state packets, four complete
interaction/live-URL packets, same-mount responsive measurements, whole response
and final DOM source, screenshot hashes, all main/popup page errors and exact
source-file/workflow identity. Failure writes the partial receipt before throwing.
Source-file hashing and Git identity lookup share that failure boundary. The real
caller is checked in isolated missing-source and gitless checkouts: both failures
preserve every successfully read source digest and the whole error before any
browser context is created. These two laws failed for absent receipts before the
review repair and passed afterward; they do not replace browser acceptance.
The early tooling hook parses the actual whole inline script, verifies module
ordering/headerless startup, and imports the real renderer without browser/native
execution. Those checks supplement the authored live-URL browser laws.
Original local visuals passed; actual ordinary locale selection was a full
document navigation, while same-mount selection uses the real SPA initializer.
Fresh normal Actions, protected actual merge and deployed acceptance remain
required; local preparation does not claim any of those terminal states.
The original #8493 source Check `38044754215` remains failed: initial `didOpen`
diagnostics in the unchanged mixed-prop rename law returned `typecheck-timed-out`
instead of `[]`. Its cause remains unknown; this receipt repair does not resolve
that observation or change the original fixture, oracle or deadline.
