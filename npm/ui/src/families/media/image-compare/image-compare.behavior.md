# ImageCompare Behavior Contract

Normative state x input -> outcome table for `image-compare-root.vue`,
`image-compare-before.vue`, `image-compare-after.vue`, `image-compare-label.vue`,
and `image-compare-handle.vue` (`@vizejs/ui/image-compare`). Every row is proven
by the named test.

The root owns the divider position (`0`–`100`, percent) and publishes it as
`--vize-ui-image-compare-position`. No CSS ships; a typical recipe stacks both
images and clips the "before" side:

```css
[data-vize-ui="image-compare-root"] {
  position: relative;
}
[data-vize-ui="image-compare-before"],
[data-vize-ui="image-compare-after"] {
  position: absolute;
  inset: 0;
}
[data-vize-ui="image-compare-before"] {
  z-index: 1;
  clip-path: inset(0 calc(100% - var(--vize-ui-image-compare-position)) 0 0);
}
[data-vize-ui="image-compare-handle"] {
  position: absolute;
  inset-block: 0;
  left: var(--vize-ui-image-compare-position);
}
```

The handle follows the WAI-ARIA slider pattern. Arrow keys move the divider in
the pressed direction: Right/Left follow the reading direction and Up/Down move
a vertical divider up and down (its value is measured from the top edge).

| ID   | State              | Input                              | Outcome                                                                                                                                    | Evidence                                                                               |
| ---- | ------------------ | ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------- |
| IC1  | default            | render                             | root publishes the position variable and data hooks; the handle is a focusable `slider` with min/max/now/valuetext and orientation         | `renders slider semantics, position custom property, parts, and slot state`            |
| IC2  | focused handle     | Arrow / Page / Home / End          | moves by `step`, `pageStep`, or to an end; clamps at 0 and 100; other keys pass through; emits `change` (`keyboard`) and `commit`          | `keyboard follows the slider pattern with step, page step, Home, and End`              |
| IC3  | RTL / vertical     | keys and pointer                   | RTL measures from the right edge and mirrors horizontal arrows; vertical measures from the top and maps Up/Down                            | `right-to-left and vertical dividers map keys and pointers to the visual direction`    |
| IC4  | drag mode          | primary pointer down / move / up   | captures the pointer, focuses the handle, tracks only that pointer, marks `dragging`, and emits `commit` on release; other buttons ignored | `pointer drags move the divider with capture, focus the handle, and commit on release` |
| IC5  | hover mode         | mouse move / touch                 | the divider follows hovering mouse and pen pointers without pressing; touch and presses never drag                                         | `hover mode follows mouse pointers without pressing and ignores touch`                 |
| IC6  | unmeasured root    | pointer down                       | zero-size roots ignore pointers                                                                                                            | `unmeasured roots ignore pointers`                                                     |
| IC7  | controlled         | key press / prop change            | emits the requested position while the controlled value wins; out-of-range values clamp                                                    | `controlled position wins until the parent accepts the request`                        |
| IC8  | disabled           | keys / pointer / API               | the handle leaves the tab order with `aria-disabled`; user input is ignored; `setPosition()` still applies                                 | `disabled roots leave the tab order and ignore keys and pointers`                      |
| IC9  | `messages` / names | render                             | handle name and value text come from typed `messages`; `ariaLabel` / `ariaLabelledby` / `ariaDescribedby` override                         | `messages and explicit names localize the handle`                                      |
| IC10 | exposed instance   | read / `setPosition()` / `focus()` | exposes position, orientation, state, element; requests snap to `step` and report whether they changed                                     | `exposes typed state and imperative position and focus controls`                       |
| IC11 | missing provider   | setup                              | compound parts fail closed with the shared context diagnostic                                                                              | `compound parts require a matching root provider`                                      |
| IC12 | pure helpers       | normalize / pointer / key          | clamping, snapping, pointer mapping per axis and direction, and slider key mapping are deterministic                                       | `image-compare-value.test.ts`                                                          |
| IC13 | SSR                | isolated requests                  | markup is byte-identical, with the position variable and slider attributes rendered on the server                                          | `renders byte-identical image compare markup across isolated SSR requests`             |
| IC14 | SSR / hydration    | hydrate                            | server markup hydrates without warnings or node replacement                                                                                | `hydrates image compare markup without warnings or node replacement`                   |
| IC15 | types              | compile                            | unions, slot state, messages, and exposes are closed and read-only                                                                         | `image-compare.types.test-d.ts`                                                        |

Pointer listeners attach on mount, so server markup carries no handlers.
