# PanZoom Behavior Contract

Normative state x input -> outcome table for `pan-zoom-root.vue`,
`pan-zoom-viewport.vue`, `pan-zoom-content.vue`, `pan-zoom-zoom-in.vue`,
`pan-zoom-zoom-out.vue`, `pan-zoom-reset.vue`, `pan-zoom-fit.vue`, and
`pan-zoom-status.vue` (`@vizejs/ui/pan-zoom`). Every row is proven by the named test.

The transform maps content to viewport pixels: `viewport = content * scale + (x, y)`.
Every change passes through the scale limits and `bounds` before it is requested.
`PanZoomContent` applies `transform: translate(x, y) scale(s)` with
`transform-origin: 0 0` inline (positioning mechanics) and publishes
`--vize-ui-pan-zoom-x`, `--vize-ui-pan-zoom-y`, and `--vize-ui-pan-zoom-scale`.
Consumers own the clipping box, e.g. `[data-vize-ui="pan-zoom-viewport"] { overflow: hidden }`.

| ID   | State              | Input                             | Outcome                                                                                                                                      | Evidence                                                                                |
| ---- | ------------------ | --------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| PZ1  | idle               | render / mount                    | focusable `role="group"` viewport with localized role description, inline transform + custom properties, labelled controls, polite status    | `renders a labelled focusable viewport, transformed content, controls, and live status` |
| PZ2  | idle               | one pointer drag                  | pans by the pointer delta (`pointer`), `data-state="panning"`, emits `transformStart`/`transformEnd`; secondary mouse buttons are ignored    | `single-pointer drags pan with start and end events`                                    |
| PZ3  | two pointers       | spread / move / release one       | scales by the distance ratio around the midpoint (`pinch`), then continues as a pan from the remaining pointer                               | `two pointers pinch around their midpoint and fall back to panning`                     |
| PZ4  | `wheelMode="zoom"` | wheel / ctrl-wheel                | zooms around the cursor (line and page deltas normalized); trackpad pinches use a higher sensitivity; one start/end pair per burst           | `wheel zooms around the cursor, trackpad pinches zoom, and bursts settle once`          |
| PZ5  | other wheel modes  | wheel                             | `pan` scrolls the content (ctrl still zooms); `zoom-with-ctrl` leaves unmodified wheels to the page                                          | `wheel modes pan or defer to page scrolling without modifiers`                          |
| PZ6  | `doubleClickZoom`  | double-click (+Shift)             | zooms one step in (out) around the pointer; disabled by `doubleClickZoom=false`                                                              | `double-click zooms in one step at the pointer and Shift zooms out`                     |
| PZ7  | focused viewport   | arrows / `+` `=` `-` / `0` / Home | arrows pan by `panStep` (Shift ×4), zoom keys zoom around the center, `0` resets, Home fits; keys from content or with modifiers are ignored | `keyboard pans, zooms around the center, resets, and fits`                              |
| PZ8  | controls           | click                             | zoom one step around the center, reset, or fit; zoom buttons disable at the effective limits; the status announces the settled level         | `buttons zoom by one step, reset, fit, and disable at the scale limits`                 |
| PZ9  | `bounds`           | any change                        | `contain` centers smaller axes and blocks gaps; `cover` raises the minimum scale to cover the viewport                                       | `bounds constrain every change`                                                         |
| PZ10 | controlled         | any change                        | emits the requested transform while rendering the controlled one until the parent accepts it                                                 | `controlled transforms win until the parent accepts the request`                        |
| PZ11 | `disabled`         | pointer / wheel / keys / clicks   | every interaction is ignored, the viewport leaves the tab order, buttons disable; the imperative API still applies                           | `disabled roots ignore input but keep the imperative API`                               |
| PZ12 | controls           | click with `preventDefault()`     | the transform is unchanged                                                                                                                   | `controls honor preventDefault from click listeners`                                    |
| PZ13 | `messages`         | render                            | role description, button labels, and the zoom announcement are localized                                                                     | `messages localize the role description, labels, and zoom announcement`                 |
| PZ14 | exposed instance   | imperative calls                  | `setTransform`, `zoomTo`, `zoomIn`, `zoomOut`, `panBy`, `reset`, `fit` report whether the transform changed                                  | `exposes typed state and imperative transform controls`                                 |
| PZ15 | missing provider   | setup                             | compound parts fail closed with the shared context diagnostic                                                                                | `compound parts require a matching root provider`                                       |
| PZ16 | transform math     | pure helpers                      | zoom anchoring, clamping, contain/cover/region bounds, fit, wheel normalization, and pinch math are deterministic                            | `pan-zoom-transform.test.ts`                                                            |
| PZ17 | SSR                | isolated requests                 | markup is byte-identical, includes the initial transform, and omits client-only `touch-action`                                               | `renders byte-identical pan-zoom markup across isolated SSR requests`                   |
| PZ18 | SSR / hydration    | hydrate                           | server markup hydrates without warnings or node replacement                                                                                  | `hydrates pan-zoom markup without warnings or node replacement`                         |
| PZ19 | types              | compile                           | transforms, bounds, modes, sources, messages, and exposes are closed and read-only                                                           | `pan-zoom.types.test-d.ts`                                                              |

Listeners, `touch-action: none`, and pointer capture are client-only. Momentum and a
minimap are intentionally out of scope.
