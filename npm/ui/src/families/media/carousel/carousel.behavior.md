# Carousel Behavior Contract

Normative state x input -> outcome table for `carousel-root.vue`,
`carousel-viewport.vue`, `carousel-slide.vue`, `carousel-previous.vue`,
`carousel-next.vue`, `carousel-indicator-group.vue`, `carousel-indicator.vue`, and
`carousel-autoplay-toggle.vue` (`@vizejs/ui/carousel`). Every row is proven by
the named test.

The carousel follows the WAI-ARIA APG carousel pattern on top of native scroll
snapping: the viewport is the consumer-styled scroll container, the active index
is the source of truth, and the track scrolls to the active slide. User scrolls
(touch swipes, trackpads, scrollbars) settle on the nearest slide without being
corrected. `slideCount` and each slide's `index` are declared so labels,
navigation availability, and indicators are correct on the server.

Minimal consumer CSS: `[data-vize-ui="carousel-viewport"] { display: flex;
overflow: auto; scroll-snap-type: x mandatory }` and
`[data-vize-ui="carousel-slide"] { flex: 0 0 100%; scroll-snap-align: start }`.

| ID  | State                    | Input                                  | Outcome                                                                                                                                            | Evidence                                                                                |
| --- | ------------------------ | -------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| C1  | default                  | render                                 | `section` with `aria-roledescription="carousel"`, focusable polite viewport, `group`/`slide` slides labelled "n of N", wired controls and tablist  | `renders the carousel pattern with deterministic ids, slide labels, and wired controls` |
| C2  | non-looping              | Previous / Next                        | moves one slide, smooth-scrolls the viewport so the slide aligns with its start edge, and disables the control at each end                         | `previous and next move one slide, scroll the track, and stop at the ends`              |
| C3  | `loop`                   | Previous at start / Next at end        | wraps to the other end; a single slide never wraps                                                                                                 | `loop wraps navigation at both ends`                                                    |
| C4  | controlled `modelValue`  | navigation                             | emits the requested index while rendering the controlled index; out-of-range values clamp                                                          | `controlled index wins until the parent accepts the request`                            |
| C5  | indicators               | click / Arrow / Home / End             | selects the slide, keeps one roving tab stop on the active indicator, and moves focus with the selection                                           | `indicators select slides and move roving focus with arrow, Home, and End keys`         |
| C6  | focused viewport         | Arrow / Home / End                     | arrows follow orientation and `dir`; keys from slide content are ignored                                                                           | `the focused viewport maps arrow keys by orientation and reading direction`             |
| C7  | user scroll              | `scroll` quiet period or `scrollend`   | the slide nearest the viewport start becomes active with reason `scroll`, without a corrective scroll                                              | `user scrolling settles on the nearest slide without re-scrolling`                      |
| C8  | programmatic scroll      | own `scrollend` / reduced motion       | initial `defaultValue` scrolls instantly; programmatic settles are ignored; reduced motion scrolls with `behavior: "auto"`                         | `programmatic scrolls ignore their own settle and respect reduced motion`               |
| C9  | multiple slides per view | scrolled to the end                    | Next disables once the track cannot scroll further                                                                                                 | `scroll edges disable next when multiple slides fill the viewport`                      |
| C10 | mouse drag               | pointerdown / move / up                | moves the track with snapping suspended, pages past `dragThreshold`, re-snaps, and swallows the trailing click; touch and `draggable=false` skip   | `mouse drags move the track, page past the threshold, and suppress the trailing click`  |
| C11 | short drag               | pointer release below the threshold    | re-aligns to the starting slide without a change event                                                                                             | `short drags snap back to the starting slide`                                           |
| C12 | autoplay playing         | interval elapses                       | advances with reason `autoplay`, rewinds after the last slide, and sets the viewport live region to `off`                                          | `autoplay advances on an interval, rewinds at the end, and silences the live region`    |
| C13 | autoplay playing         | hover / pointer focus / keyboard focus | mouse hover pauses; pointer-initiated focus does not stop; keyboard focus stops (`focusBehavior="stop"`) or pauses (`"pause"`); touch never hovers | `hover and pointer focus pause rotation while keyboard focus stops it`                  |
| C14 | autoplay playing         | reduced motion / hidden document       | `prefers-reduced-motion` and `document.visibilityState="hidden"` pause rotation unless `respectReducedMotion=false`                                | `reduced motion and hidden documents pause rotation`                                    |
| C15 | controlled `playing`     | toggle / single slide                  | emits `update:playing` while the controlled intent wins; one slide never rotates and disables the toggle                                           | `controlled playing and single-slide carousels keep rotation stopped`                   |
| C16 | measured visibility      | intersection changes                   | slides under half visible become `inert` (never the active slide) and expose `data-in-view`                                                        | `slides measured out of view become inert while the active slide stays reachable`       |
| C17 | controls                 | click with `preventDefault()`          | Previous, Next, indicators, and the autoplay toggle leave state unchanged                                                                          | `controls honor preventDefault from click listeners`                                    |
| C18 | exposed instance         | read / imperative calls                | exposes state and `scrollTo`, `scrollNext`, `scrollPrev`, `play`, `stop`, each reporting whether state changed                                     | `exposes typed state and imperative navigation and rotation controls`                   |
| C19 | missing provider         | setup                                  | compound parts fail closed with the shared context diagnostic                                                                                      | `compound parts require a matching root provider`                                       |
| C20 | geometry                 | pure helpers                           | index clamping/wrapping, ltr/rtl/vertical offsets, nearest-slide selection, scroll edges, and drag paging are deterministic                        | `carousel-geometry.test.ts`                                                             |
| C21 | SSR                      | isolated requests                      | markup is byte-identical, reflects `defaultValue` and autoplay, and never renders `inert`                                                          | `renders byte-identical carousel markup across isolated SSR requests`                   |
| C22 | SSR / hydration          | hydrate                                | server markup hydrates without warnings or node replacement                                                                                        | `hydrates carousel markup without warnings or node replacement`                         |
| C23 | types                    | compile                                | states, reasons, slot state, and exposes are closed and read-only                                                                                  | `carousel.types.test-d.ts`                                                              |
| C24 | `messages`               | render                                 | role descriptions and slide/indicator names come from the typed `messages` prop; omitted entries keep the English WAI-ARIA wording                 | `messages localize role descriptions and slide and indicator names`                     |

Timers and listeners are client-only: they start in `onMounted` and stop on
unmount, so server rendering never schedules rotation.
