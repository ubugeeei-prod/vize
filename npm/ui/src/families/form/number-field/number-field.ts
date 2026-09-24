/** Locale-aware APG spinbutton with formatting, stepping, press-and-hold triggers, and form association. */
export { default as NumberField } from "./number-field.vue";
/** Spinbutton text input for a NumberField. */
export { default as NumberFieldInput } from "./number-field-input.vue";
/** Press-and-hold trigger that increases a NumberField. */
export { default as NumberFieldIncrement } from "./number-field-increment.vue";
/** Press-and-hold trigger that decreases a NumberField. */
export { default as NumberFieldDecrement } from "./number-field-decrement.vue";
export { createNumberFieldParser } from "./number-field-parser.ts";
export type {
  NumberFieldFormatOptions,
  NumberFieldParser,
  NumberFieldPartialOptions,
} from "./number-field-parser.ts";
export {
  NUMBER_FIELD_DEFAULT_PERCENT_STEP,
  NUMBER_FIELD_DEFAULT_STEP,
  NUMBER_FIELD_LARGE_STEP_MULTIPLIER,
  canStepNumberFieldValue,
  clampNumberFieldValue,
  getNumberFieldState,
  normalizeNumberFieldBounds,
  snapNumberFieldValue,
  stepNumberFieldValue,
} from "./number-field-state.ts";
export type { NumberFieldBounds, NumberFieldBoundsOptions } from "./number-field-state.ts";
export type {
  NumberFieldAriaInvalid,
  NumberFieldChangeSource,
  NumberFieldEmits,
  NumberFieldExpose,
  NumberFieldInputEmits,
  NumberFieldInputProps,
  NumberFieldProps,
  NumberFieldSlotState,
  NumberFieldSlots,
  NumberFieldState,
  NumberFieldStepDirection,
  NumberFieldTriggerProps,
  NumberFieldTriggerSlotState,
  NumberFieldTriggerSlots,
  NumberFieldValue,
} from "./number-field-types.ts";
