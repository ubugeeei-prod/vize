import { inputMaskRuntimeFixture } from "./input-mask/runtime-conformance-input-mask-fixtures.ts";
import { numberFieldRuntimeFixtures } from "./number-field/runtime-conformance-number-field-fixtures.ts";
import { rangeSliderRuntimeFixtures } from "./range-slider/runtime-conformance-range-slider-fixtures.ts";
import type { RuntimeFixture } from "../../conformance/runtime-conformance-fixtures.ts";

/** SSR and hydration fixtures for every extended form-input SFC. */
export const formInputRuntimeFixtures: readonly RuntimeFixture[] = [
  inputMaskRuntimeFixture,
  ...numberFieldRuntimeFixtures,
  ...rangeSliderRuntimeFixtures,
];
