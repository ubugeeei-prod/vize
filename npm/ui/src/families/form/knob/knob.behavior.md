# Knob behavior contract

Normative state x input -> outcome table for `knob.vue` (`@vizejs/ui/knob`), a
rotary control with WAI-ARIA APG slider semantics. Angles are degrees clockwise
from 12 o'clock over a sweep (default -135° to 135°). Every row is proven by the
named test in `knob.test.ts` or `knob-ssr.test.ts`; compile-only assertions live
in `knob.types.test-d.ts`.

| #   | State             | Input                     | Outcome                                                                                                                    | Proven by                                                                    |
| --- | ----------------- | ------------------------- | -------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| K1  | geometry          | angle <-> value           | values map linearly onto the sweep; pointer angles in the dead zone resolve to the nearer end (no jump across ends)        | `maps angles and values with a dead zone that never jumps across ends`       |
| K2  | named             | render                    | focusable `role="slider"` with `aria-valuenow/min/max/valuetext`, `--vize-knob-angle`/`--vize-knob-percent`, hidden value  | `renders an APG slider with angle hooks and a hidden form value`             |
| K3  | focused           | Arrow / Page / Home / End | Up/Right increase, Down/Left decrease, pages use `largeStep`, Home/End jump to bounds; each distinct change commits        | `keyboard steps, pages, and jumps to bounds, committing each change`         |
| K4  | idle              | pointer drag              | press focuses and maps the pointer angle around the center; moves update the value; release commits once                   | `dragging rotates around the center and commits once on release`             |
| K5  | wheel / locked    | wheel, keys, pointer      | wheel is opt-in and needs focus; disabled and read-only knobs ignore input and emit nothing                                | `wheel is opt-in and needs focus; disabled and read-only knobs ignore input` |
| K6  | controlled / form | keys, reset, API          | controlled values win; the hidden input submits and form reset restores `defaultValue`; `setValue` snaps; `reset`, `focus` | `controlled values, form reset, and the imperative API`                      |
| K7  | SSR / hydration   | isolated requests         | byte-identical markup with the CSS angle and no `NaN`/`Infinity`; hydration without diagnostics                            | `renders byte-identical rotary markup and hydrates without mismatches`       |

## Public extension contract

| Surface               | Contract                                                                       |
| --------------------- | ------------------------------------------------------------------------------ |
| Parts                 | `root` (the `role="slider"` element; the slot draws the face).                 |
| Data attributes       | `data-vize-ui="knob"`, `data-state` (`idle`/`dragging`/`readonly`/`disabled`). |
| CSS custom properties | `--vize-knob-angle` (deg) and `--vize-knob-percent`.                           |

The subpath is tree-shakable and ships no CSS; those package contracts are
pinned by `distribution.test.ts`, `check:size`, and `check:tree-shaking`.
