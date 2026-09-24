# AnglePicker behavior contract

Normative state x input -> outcome table for `angle-picker.vue`
(`@vizejs/ui/angle-picker`), a full-circle angle input with WAI-ARIA APG slider
semantics that reuses Knob's rotary geometry and drag handling. Every row is
proven by the named test in `angle-picker.test.ts` or `../knob/knob-ssr.test.ts`;
compile-only assertions live in `angle-picker.types.test-d.ts`.

| #   | State                       | Input                     | Outcome                                                                                                                  | Proven by                                                              |
| --- | --------------------------- | ------------------------- | ------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------- |
| A1  | seeded                      | render                    | `role="slider"` with 0 to `360 - step`, wrapped value, "N degrees" value text, `--vize-angle-picker-angle`, hidden value | `renders a full-circle slider with degree value text and a CSS angle`  |
| A2  | focused                     | Arrow / Page / Home / End | steps wrap around the circle; Home is 0°, End is the last step; each change commits                                      | `keyboard steps wrap around the circle`                                |
| A3  | idle                        | pointer drag              | the pointer angle around the center becomes the value, snapped to `step`; release commits once                           | `pointer rotation follows the pointer and snaps to the step`           |
| A4  | controlled / disabled / API | keys, expose              | controlled values win; disabled pickers ignore keys; `setValue` wraps; `reset` restores the default                      | `controlled, disabled, and imperative behavior`                        |
| A5  | SSR                         | isolated requests         | byte-identical markup and hydration without diagnostics                                                                  | `renders byte-identical rotary markup and hydrates without mismatches` |

The subpath is tree-shakable and ships no CSS; those package contracts are
pinned by `distribution.test.ts`, `check:size`, and `check:tree-shaking`.
