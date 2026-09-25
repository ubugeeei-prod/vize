# Lightbox Behavior Contract

Normative state x input -> outcome table for `lightbox-root.vue`,
`lightbox-trigger.vue`, `lightbox-content.vue`, `lightbox-item.vue`,
`lightbox-image.vue`, `lightbox-previous.vue`, `lightbox-next.vue`,
`lightbox-close.vue`, `lightbox-counter.vue`, `lightbox-thumbnails.vue`, and
`lightbox-thumbnail.vue` (`@vizejs/ui/lightbox`).
Every row is proven by the named test.

The viewer is built on the Dialog family: `LightboxRoot` drives a `DialogRoot`,
and `LightboxContent` renders `DialogPortal` + `DialogContent`. It therefore
inherits modal focus containment, focus return, Escape, outside dismissal, inert
outside content, and scroll locking. `LightboxRoot` is generic over `items`, so
the root slot's `item` keeps the consumer's item type.

| ID  | State            | Input                            | Outcome                                                                                                                                         | Evidence                                                                              |
| --- | ---------------- | -------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| L1  | closed           | trigger click / Close            | opens a labelled modal dialog at the trigger's index (reason `trigger`); closing returns focus to that trigger                                  | `renders closed triggers that open the dialog at their item and restore focus`        |
| L2  | open             | Previous / Next                  | moves one item, disables at the ends, wraps with `loop`                                                                                         | `previous and next navigate, disable at the ends, and wrap with loop`                 |
| L3  | open             | Arrow / Home / End               | arrows follow `dir`; Home/End jump; keys in text fields or with modifiers are ignored                                                           | `arrow, Home, and End keys navigate with reading direction and skip text fields`      |
| L4  | open             | Escape                           | the dialog layer closes the viewer and emits `update:open`                                                                                      | `Escape closes through the dialog layer`                                              |
| L5  | open             | touch/pen swipe                  | horizontal swipes past `swipeThreshold` navigate (RTL aware); a downward swipe closes unless disabled; mouse and cancelled gestures are ignored | `touch swipes navigate by direction and a downward swipe closes`                      |
| L6  | open             | thumbnails                       | a roving tablist selects items on click and arrows/Home/End, moves focus, and is not double-handled by the stage                                | `thumbnails form a roving tablist that selects and focuses items`                     |
| L7  | controlled       | `v-model:open` / `v-model:index` | requests are emitted while controlled values win; out-of-range indexes clamp                                                                    | `controlled open and index win until the parent accepts the request`                  |
| L8  | open             | index changes                    | `getPreloadSrc` warms `preload` neighbours once each, on the client only, never while closed                                                    | `preloads neighbouring images on the client while open`                               |
| L9  | messages         | render                           | every accessible name, the counter, and the slide role description come from typed `messages`; `ariaLabelledby` replaces the dialog label       | `localized messages label every control`                                              |
| L10 | controls / API   | `preventDefault()` / expose      | controls honor `preventDefault()`; the instance exposes typed state and `openAt`/`close`/`goTo`/`next`/`previous`                               | `controls honor preventDefault and exposes typed imperative controls`                 |
| L11 | portal           | open                             | content teleports to `body` by default                                                                                                          | `teleports the viewer to the document body by default`                                |
| L12 | missing provider | setup                            | parts fail closed with a context diagnostic                                                                                                     | `compound parts require a matching root provider`                                     |
| L13 | pure helpers     | state                            | index resolution, preload neighbours, swipe classification, and messages are deterministic                                                      | `resolves indexes, preload neighbours, swipes, and messages`                          |
| L14 | SSR              | isolated requests                | closed and open markup are byte-identical                                                                                                       | `renders byte-identical closed and open lightbox markup across isolated SSR requests` |
| L15 | SSR / hydration  | hydrate                          | open server markup hydrates without warnings or node replacement                                                                                | `hydrates open lightbox markup without warnings or node replacement`                  |
| L16 | types            | compile                          | item types flow into slots and exposes; reasons, swipes, and messages are closed                                                                | `lightbox.types.test-d.ts`                                                            |

The counter is a polite live region, so position changes are announced.
`LightboxImage` wraps the Image family: eager loading, safe source chains, a
delayed placeholder, and a fallback.
