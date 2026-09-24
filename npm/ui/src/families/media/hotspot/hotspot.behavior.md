# Hotspot Behavior Contract

Normative state x input -> outcome table for `hotspot-root.vue`,
`hotspot-image.vue`, `hotspot-marker.vue`, `hotspot-content.vue`, and
`hotspot-area.vue` (`@vizejs/ui/hotspot`). Every row is proven by the named test.

Markers are positioned in image space: each HotspotMarker publishes
`--vize-ui-hotspot-x` / `--vize-ui-hotspot-y` percentages, so a consumer rule
like `position: absolute; left: var(--vize-ui-hotspot-x); top: var(--vize-ui-hotspot-y)`
keeps markers attached to the image at every size. Each marker is a Popover
(`PopoverRoot` + `PopoverTrigger` button), and HotspotContent wraps
`PopoverContent`, inheriting Escape, outside dismissal, positioning, and focus
return. HotspotArea draws a region on a `viewBox="0 0 100 100"` SVG overlay.

| ID  | State               | Input                           | Outcome                                                                                                                                     | Evidence                                                                         |
| --- | ------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| H1  | default             | render                          | markers render as named buttons with position variables and states; the image keeps safe `src`/`alt`; disabled markers are disabled buttons | `renders positioned marker buttons, a safe image, regions, and typed slot data`  |
| H2  | exclusive (default) | marker click                    | opens that marker's content, closes the previous one, toggles closed on a second click; emits `update:active` and `activeChange`            | `clicking a marker opens its content and exclusive roots close the previous one` |
| H3  | open content        | Escape                          | closes the content and returns focus to its marker                                                                                          | `Escape closes the open content and returns focus to its marker`                 |
| H4  | `exclusive=false`   | clicks / `open()` / `close()`   | several markers stay open; `active` is the latest; closing it restores the previous one; `close()` without an id closes all                 | `non-exclusive roots keep several markers open and track the latest as active`   |
| H5  | focused marker      | Arrow keys                      | focus moves to the nearest enabled marker inside a 90° cone in that direction; modified keys and dead ends pass through                     | `arrow keys move focus to the spatially nearest enabled marker`                  |
| H6  | controlled `active` | click / prop change             | emits the request while the controlled marker stays open until the parent accepts it                                                        | `controlled active wins until the parent accepts the request`                    |
| H7  | regions             | click / Enter / hit test        | button regions toggle with `aria-pressed`; link regions render sanitized `href`; `hitTest()` returns areas containing a point               | `regions toggle as buttons, render safe links, and support hit testing`          |
| H8  | unsafe / disabled   | render / click                  | unsafe image sources and links are dropped; disabled regions leave the tab order and ignore activation                                      | `unsafe links and images are not rendered and disabled regions stay inert`       |
| H9  | region listener     | `activate` + `preventDefault()` | the toggle is cancelled                                                                                                                     | `region activate listeners can cancel the toggle`                                |
| H10 | missing provider    | setup                           | markers and areas require HotspotRoot; content requires HotspotMarker                                                                       | `compound parts require matching providers`                                      |
| H11 | pure geometry       | helpers                         | even-odd polygon test, inclusive rect/circle tests, point formatting, clamping, directional search, and href sanitizing are deterministic   | `hotspot-geometry.test.ts`                                                       |
| H12 | SSR                 | isolated requests               | markup is byte-identical, including generated marker ids, the open default marker, and the SVG region                                       | `renders byte-identical hotspot markup across isolated SSR requests`             |
| H13 | SSR / hydration     | hydrate                         | server markup hydrates without warnings or node replacement                                                                                 | `hydrates hotspot markup without warnings or node replacement`                   |
| H14 | types               | compile                         | hotspot payloads flow through `HotspotSlotState<Data>`; states, shapes, and exposes are closed and read-only                                | `hotspot.types.test-d.ts`                                                        |

HotspotContent defaults `closeOnFocusOutside` to `false` so that focus returning
to one marker never closes another open marker.
