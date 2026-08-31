# Slider behavior contract

Normative state x input -> outcome table for `slider.vue` (`@vizejs/ui/slider`).
Every row is proven by the named mounted-DOM or SSR test; a row without a
passing test is a contract violation.

| #   | State                 | Input         | Outcome                                                                                                                                       | Proven by                                                                  |
| --- | --------------------- | ------------- | --------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| S1  | named, bounded        | render        | headless root with native `<input type="range">`, deterministic `id`, form attributes, ARIA field state, parts, data attributes, and CSS vars | `renders a named native range input with form and accessibility hooks`     |
| S2  | uncontrolled          | native input  | clamps/snaps the next value, emits `update:modelValue` before `input`, updates form value, data attributes, CSS vars, slot state, and expose  | `uncontrolled slider updates native form value and slot state`             |
| S3  | controlled            | native input  | emits the requested value and `input`; rendered value returns to `modelValue` until the parent accepts the update                             | `controlled value wins until the parent accepts the request`               |
| S4  | uncontrolled, seeded  | form reset    | `defaultValue` seeds initial state and form reset restores it without request-global state                                                    | `defaultValue seeds state and native form reset restores it`               |
| S5  | focusable             | keyboard      | native range keyboard behavior remains available for editable sliders while the exposed API can focus and step the value                      | `exposes focus, setValue, stepUp, stepDown, reset, and normalized state`   |
| S6  | disabled              | Tab / form    | native `disabled`, `data-state="disabled"`, no sequential focus, and no submitted form value                                                  | `disabled and read-only sliders keep availability semantics`               |
| S7  | read-only             | pointer / key | remains focusable with `aria-readonly` and form value, while user pointer/key/input/change attempts are prevented or restored                 | `disabled and read-only sliders keep availability semantics`               |
| S8  | vertical RTL, invalid | render / SSR  | `orientation`, `dir`, `aria-orientation`, `aria-invalid`, `aria-valuetext`, and repaired numeric bounds are stable across SSR and hydration   | `renders byte-identical native slider markup across isolated SSR requests` |

## Public extension contract

| Surface               | Contract                                                                                                                                |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| Parts                 | `root` on the host `<span>`; `control` on the native range input.                                                                       |
| Data attributes       | `data-vize-ui`, `data-state`, `data-orientation`, `data-dir`, `data-value`, `data-min`, `data-max`, `data-step`, and `data-percent`.    |
| Boolean data hooks    | `data-disabled`, `data-readonly`, `data-required`, and `data-invalid` are present as `"true"` only while active.                        |
| CSS custom properties | `--vize-slider-value`, `--vize-slider-min`, `--vize-slider-max`, `--vize-slider-step`, and `--vize-slider-percent` are set on the root. |
| Slot                  | The default slot receives `SliderSlotState` for optional output, marks, and native CSS authored by consumers.                           |

The subpath remains tree-shakable and retains no packaged CSS; those package
contracts are pinned by `distribution.test.ts`, `check:size`, and
`check:tree-shaking`.
