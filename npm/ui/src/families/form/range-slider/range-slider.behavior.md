# RangeSlider behavior contract

Normative state x input -> outcome table for `range-slider.vue`,
`range-slider-track.vue`, `range-slider-range.vue`, and `range-slider-thumb.vue`
(`@vizejs/ui/range-slider`), following the WAI-ARIA APG multi-thumb slider
pattern. The single-thumb native `Slider` stays in `@vizejs/ui/slider`;
RangeSlider reuses its bound/step normalization (`slider-state.ts`) rather than
duplicating it. Every row is proven by the named test in `range-slider.test.ts`
or `range-slider-ssr.test.ts`; compile-only assertions live in
`range-slider.types.test-d.ts`.

| #   | State                 | Input                           | Outcome                                                                                                                                                     | Proven by                                                                   |
| --- | --------------------- | ------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| R1  | raw values            | normalize                       | values snap to the grid, sort ascending, keep `minStepsBetweenThumbs * step` apart, and never cross neighbors; bounds repair like Slider                    | `normalizes, sorts, spaces, and moves values without crossing neighbors`    |
| R2  | two thumbs, named     | render                          | labelled `role="group"`; each thumb is a focusable `role="slider"` whose `aria-valuemin/max` are its neighbor-limited range; hidden input per value         | `renders a labelled group of APG slider thumbs with range hooks`            |
| R3  | focused thumb         | Arrow / Page / Home / End       | arrows step, pages use `largeStep` (default `step * 10`), Home/End go to the neighbor-limited bounds; distinct results emit `change(values, i, "keyboard")` | `keyboard moves one thumb by step, page, and bounds and commits changes`    |
| R4  | RTL / vertical        | arrow keys                      | RTL mirrors Left/Right; vertical sliders increase with Up                                                                                                   | `RTL mirrors horizontal arrows and vertical sliders use up for increase`    |
| R5  | idle                  | pointer down, move, up on track | the closest thumb jumps, gains focus, drags (clamped to neighbors and track), `data-state="dragging"`; release emits one `change(..., "pointer")`           | `pointer down moves the closest thumb, drags it, and commits on release`    |
| R6  | RTL / vertical        | pointer                         | RTL measures from the right edge; vertical measures from the bottom                                                                                         | `pointer mapping honors RTL and vertical tracks`                            |
| R7  | controlled            | keyboard                        | emits the request; rendered values stay on `modelValue` until the parent accepts                                                                            | `controlled values win until the parent accepts the request`                |
| R8  | in a form             | submit / reset                  | submits every value under `name`; form reset restores `defaultValue`                                                                                        | `submits one value per thumb and restores defaults on form reset`           |
| R9  | disabled              | keyboard / pointer              | thumbs stay focusable for discovery with `aria-disabled`, ignore keyboard and pointer input, and hidden values are disabled                                 | `disabled sliders stay discoverable but ignore input and submit nothing`    |
| R10 | imperative            | expose                          | `setThumbValue`, `setValue` (normalized, thumb count follows), `focusThumb`, `reset`, and normalized state                                                  | `exposes setThumbValue, setValue, focusThumb, reset, and normalized state`  |
| R11 | part without provider | setup                           | throws the stable `VIZE_UI_CONTEXT_MISSING: RangeSlider` diagnostic                                                                                         | `thumbs and parts require a RangeSlider provider`                           |
| R12 | SSR / hydration       | isolated requests               | byte-identical markup with CSS custom properties and value text; hydration keeps nodes and thumbs stay interactive                                          | `renders byte-identical multi-thumb markup and hydrates without mismatches` |

## Public extension contract

| Surface               | Contract                                                                                                              |
| --------------------- | --------------------------------------------------------------------------------------------------------------------- |
| Parts                 | `root`, `track`, `range`, `thumb`.                                                                                    |
| Data attributes       | `data-vize-ui`, root `data-state` / `data-orientation` / `data-dir`; thumb `data-index`, `data-active`.               |
| Boolean hooks         | `data-disabled`, `data-invalid` (root), `data-active` (thumb) are `"true"` only while active.                         |
| CSS custom properties | Root: `--vize-range-slider-range-start`, `--vize-range-slider-range-end`. Thumb: `--vize-range-slider-thumb-percent`. |
| Slots                 | Root default slot receives `RangeSliderSlotState`; render one `RangeSliderThumb` per `values` entry.                  |

The subpath is tree-shakable and ships no CSS; those package contracts are
pinned by `distribution.test.ts`, `check:size`, and `check:tree-shaking`.
