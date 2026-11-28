# Color Wheel behavior contract

Normative state × input → outcome table for `src/components/color-wheel/color-wheel.vue`.
Hue 0 is the positive x axis. Screen y grows downward, so the wheel runs clockwise.
Every row is proven by `src/components/color-wheel/tests/behavior.test.ts`.

| #   | State    | Input                                        | Outcome                                                   | Proven by                                        |
| --- | -------- | -------------------------------------------- | --------------------------------------------------------- | ------------------------------------------------ |
| C1  | geometry | point on +x, +y, -x, -y                      | hues 0, 90, 180, 270                                      | `maps screen axes to a clockwise hue`            |
| C2  | hue 0    | render                                       | slider named Hue, value 0, marker at the right (`86, 50`) | `renders hue 0 at the right of the wheel`        |
| C3  | hue 0    | ArrowRight / ArrowLeft                       | 1 / 359, and the key is canceled                          | `steps hue from the arrow keys and wraps`        |
| C4  | hue 40   | Home / End                                   | 0 / 360                                                   | `sends Home to 0 and End to 360`                 |
| C5  | hue 0    | Shift+ArrowRight, then PageDown, then PageUp | 10, then 0, then 10                                       | `uses a ten-degree step for Shift and Page keys` |
| C6  | enabled  | pointer down at the right of the center      | hue 0                                                     | `reads the pointer as a clockwise hue`           |
| C7  | disabled | render, ArrowRight, pointer                  | `aria-disabled`, not tabbable, no change                  | `disabled wheel ignores keyboard and pointer`    |
| C8  | any      | default slot                                 | slot receives `hue` and an `hsl()` color                  | `passes hue and color to the slot`               |
