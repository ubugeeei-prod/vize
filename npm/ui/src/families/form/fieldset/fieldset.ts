/** Native `<fieldset>` group with legend, description, and error wiring. */
export { default as Fieldset } from "./fieldset.vue";
/** Native `<legend>` that names the Fieldset. */
export { default as FieldsetLegend } from "./fieldset-legend.vue";
/** Group-level description referenced by the Fieldset. */
export { default as FieldsetDescription } from "./fieldset-description.vue";
/** Group-level error message referenced by the Fieldset while invalid. */
export { default as FieldsetErrorMessage } from "./fieldset-error-message.vue";
export type {
  FieldsetErrorMessageSlotState,
  FieldsetSlotState,
  FieldsetState,
} from "./fieldset-types.ts";
