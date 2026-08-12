# Collection registry behavior

`collection.ts` is the shared ordered-item and logical-focus contract for listbox,
menu, tree, tabs, grid, command palette, combobox, and other compound widgets. It
is headless: consumers render elements and bind `activeKey` to roving tabindex or
`aria-activedescendant` without the registry writing DOM or owning selection.

## State × input → outcome

| State                           | Input                                                                                      | Required outcome                                                                                                                                             |
| ------------------------------- | ------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Live registry                   | Register a unique non-empty string or safe-integer key                                     | One immutable item snapshot is exposed; the consumer value keeps its exact generic type                                                                      |
| Live registry                   | Register an empty/control-bearing string, non-safe number, negative zero, or duplicate key | Registration throws a stable `VIZE_UI_COLLECTION_*` diagnostic and leaves the registry unchanged                                                             |
| No explicit `order`             | Every item element is connected in one document                                            | Items follow live DOM order, independent of setup or registration order                                                                                      |
| No explicit `order`             | SSR, pre-mount, an incomplete/disconnected item set, or different documents                | The entire collection retains deterministic registration order, avoiding a non-transitive mix of DOM and fallback ordering                                   |
| Connected or partial item roots | Relevant child, text, or accessible-label DOM mutation                                     | The connected common root and disconnected items are observed without watching the whole document; `refresh()` provides a deterministic fallback             |
| Explicit `order`                | Every item supplies a unique safe integer                                                  | Items follow ascending virtual order in SSR and the browser                                                                                                  |
| Explicit `order`                | An order is missing, duplicated, or not a safe integer                                     | Resolution throws and a rejected registration is rolled back atomically                                                                                      |
| Missing explicit `textValue`    | A rendered element is available                                                            | Text is extracted from same-root `aria-labelledby`, `aria-label`, inline content, image alt text, or input value; hidden decorative descendants are excluded |
| Any text source                 | Whitespace or decomposed Unicode is present                                                | Text is NFC-normalized, trimmed, and whitespace-collapsed without locale-specific case folding                                                               |
| `textValue=""`                  | Typeahead runs                                                                             | The item deliberately does not match typeahead                                                                                                               |
| `disabledBehavior="skip"`       | An item is disabled                                                                        | The item remains inspectable but is excluded from navigation and typeahead; direct activation-key assignment is rejected                                     |
| `disabledBehavior="focusable"`  | An item is disabled                                                                        | The item remains in focus navigation and typeahead so menu-style disabled-item semantics are possible; activation remains a higher-level policy              |
| No active key                   | Move next/first or previous/last                                                           | The first or last navigable key becomes active respectively                                                                                                  |
| Active key in the middle        | Move next/previous                                                                         | The adjacent navigable key becomes active; skipped disabled items are never selected                                                                         |
| Active key at a boundary        | Move without/with `loop`                                                                   | The key remains unchanged without looping and wraps with looping                                                                                             |
| Active key                      | Typeahead prefix or exact search                                                           | The next locale-collated match after the active item is returned; wrapping defaults to enabled for repeated-character cycling                                |
| Active item                     | Item unmounts or unregisters                                                               | Recovery synchronously chooses the next surviving navigable item, then the previous item, then `null`                                                        |
| Active item                     | Reactive disabled state becomes non-navigable                                              | The same synchronous next-then-previous recovery runs with reason `item-disabled`                                                                            |
| Vue owner scope                 | Item or registry scope stops                                                               | Registrations are removed or the registry is disposed automatically; active state cannot point at destroyed content                                          |
| Disposed registry               | A later mutation is requested                                                              | Mutation throws `VIZE_UI_COLLECTION_DISPOSED`; repeated disposal is idempotent                                                                               |

## Accessibility boundary

The registry supplies stable identity, deterministic order, disabled navigation
policy, normalized text, and mutation-safe logical focus. A concrete composite
must still apply its APG role model, accessible name and relationships,
orientation, keyboard map, focus strategy, selection semantics, live
announcements, and activation policy. Keeping those responsibilities explicit
allows the same collection to serve roving DOM focus, `aria-activedescendant`,
virtualized content, portalled content, DOM rendering, Vapor, and SSR.
