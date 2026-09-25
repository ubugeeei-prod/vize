import type { StyleValue } from "vue";

import type { KnobChangeSource, KnobExpose, KnobSlotState, KnobState } from "../knob/knob-types.ts";

/** Values accepted by the `aria-invalid` attribute. */
export type AnglePickerAriaInvalid = boolean | "grammar" | "spelling";

/** Source of a committed angle change. */
export type AnglePickerChangeSource = KnobChangeSource;

/** State published through the AnglePicker `data-state` contract. */
export type AnglePickerState = KnobState;

/** CSS custom properties authored on the AnglePicker root. */
export type AnglePickerStyle = StyleValue & {
  readonly "--vize-angle-picker-angle": string;
};

/** State exposed to AnglePicker slots (`value` and `angle` are equal, in `[0, 360)`). */
export type AnglePickerSlotState = KnobSlotState;

/** Public instance API of AnglePicker. */
export type AnglePickerExpose = KnobExpose;
