# Marquee Behavior Contract

Normative state x input -> outcome table for `marquee-root.vue`,
`marquee-content.vue`, and `marquee-pause-button.vue` (`@vizejs/ui/marquee`).
Every row is proven by the named test.

Animation uses consumer CSS keyframes driven by measured custom properties rather
than the Web Animations API. The markup and the paused state stay declarative, so
they are identical on the server and after hydration. Reduced-motion media
queries also work in plain CSS, no JavaScript frame loop runs, and pausing is just
`animation-play-state` keyed on `data-state`. The root publishes:

- `--vize-ui-marquee-distance`: the distance between two copies, gap included.
- `--vize-ui-marquee-duration`: `distance / speed`.
- `--vize-ui-marquee-copies`: the number of copies.

Recipe:

```css
[data-vize-ui="marquee-root"] {
  overflow: hidden;
}
[data-vize-ui="marquee-track"] {
  display: flex;
  width: max-content;
  animation: vize-marquee var(--vize-ui-marquee-duration, 0s) linear infinite;
}
[data-vize-ui="marquee-root"][data-state="paused"] [data-vize-ui="marquee-track"] {
  animation-play-state: paused;
}
[data-direction="right"] [data-vize-ui="marquee-track"],
[data-direction="down"] [data-vize-ui="marquee-track"] {
  animation-direction: reverse;
}
@keyframes vize-marquee {
  to {
    translate: calc(-1 * var(--vize-ui-marquee-distance, 0px));
  }
}
```

| ID  | State            | Input                     | Outcome                                                                                                                             | Evidence                                                                             |
| --- | ---------------- | ------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| M1  | default          | render                    | `role="marquee"` region with a deterministic id; the first copy is exposed, duplicates are `aria-hidden` and `inert`                | `renders an accessible marquee region with one exposed copy and inert duplicates`    |
| M2  | measured         | mount / resize            | distance, duration, and copy count derive from measured copies; `repeat="auto"` fills the viewport plus one copy                    | `measures copies to publish distance, duration, and a viewport-filling repeat count` |
| M3  | vertical         | `direction="up"` / resize | measurement uses the block axis and re-runs on resize                                                                               | `vertical directions measure along the block axis and resize re-measures`            |
| M4  | running          | hover / keyboard focus    | mouse hover and focus inside pause with reasons `hover`/`focus`; touch never hovers; both are configurable                          | `hover and keyboard focus pause and resume the animation`                            |
| M5  | running          | pause button              | toggles the play intent (WCAG 2.2.2), swapping its accessible name; `preventDefault()` keeps the intent                             | `the pause button toggles the play intent per WCAG 2.2.2`                            |
| M6  | reduced motion   | mount / play              | `prefers-reduced-motion` pauses with reason `reduced-motion` until the user presses play; opt out with `respectReducedMotion=false` | `reduced motion keeps the marquee paused until the user opts in`                     |
| M7  | controlled       | `v-model:playing` / API   | the parent owns the intent; messages localize the button; the instance exposes state and `play`/`pause`/`toggle`                    | `controlled playing, localized messages, and the exposed instance`                   |
| M8  | missing provider | setup                     | parts fail closed with the shared context diagnostic                                                                                | `compound parts require a matching root provider`                                    |
| M9  | pure helpers     | geometry                  | copy counts clamp to 2–32, durations round to milliseconds, and styles omit unmeasured values                                       | `derives orientation, copy counts, durations, and custom properties`                 |
| M10 | SSR              | isolated requests         | markup is byte-identical with two copies and no duration until measured                                                             | `renders byte-identical marquee markup across isolated SSR requests`                 |
| M11 | SSR / hydration  | hydrate                   | server markup hydrates without warnings or node replacement                                                                         | `hydrates marquee markup without warnings or node replacement`                       |
| M12 | types            | compile                   | directions, states, reasons, messages, and exposes are closed and read-only                                                         | `marquee.types.test-d.ts`                                                            |

Copies repeat the slot, so ids inside marquee content would be duplicated;
avoid them.
