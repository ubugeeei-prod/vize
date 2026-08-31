<script setup lang="ts">
import { computed, toRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { fieldContext } from "./field-context.ts";
import { useFieldWiring } from "../field-wiring/field-wiring-runtime.ts";
import { useFormField } from "../form/form-runtime.ts";
import type {
  FieldControlProps,
  FieldLabelProps,
  FieldTextProps,
} from "../field-wiring/field-wiring-types.ts";
import type { FieldRootExpose, FieldRootSlotState } from "./field-types.ts";
import type { FormFieldError } from "../form/form-types.ts";
import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";

const {
  as = "div",
  id = undefined,
  name,
  errors = [],
  invalid = undefined,
  hasDescription = false,
  hasErrorMessage = true,
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "div"
   */
  readonly as?: PrimitiveAs;

  /**
   * Consumer-owned control id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Normalized HTML form field name.
   *
   * @default required
   */
  readonly name: string;

  /**
   * Full normalized form error list produced by the form helpers.
   *
   * @default []
   */
  readonly errors?: readonly FormFieldError[];

  /**
   * Consumer-owned invalid override. `undefined` derives invalid state from matching errors.
   *
   * @default undefined
   */
  readonly invalid?: boolean;

  /**
   * Whether a FieldDescription is rendered and should join `aria-describedby`.
   *
   * @default false
   */
  readonly hasDescription?: boolean;

  /**
   * Whether a FieldErrorMessage is rendered while invalid and should be referenced.
   *
   * @default true
   */
  readonly hasErrorMessage?: boolean;
}>();

const emit = defineEmits<{
  /** Fired after the derived invalid boolean changes. */
  "invalid-change": [invalid: boolean, errors: readonly FormFieldError[]];
}>();

defineSlots<{
  /** Renders the composed field body with ids, ARIA props, and normalized field state. */
  default(props: FieldRootSlotState): unknown;
}>();

const element = useTemplateRef<PrimitiveElement>("element");
const formField = useFormField({
  errors: toRef(() => errors),
  name: toRef(() => name),
});
const isInvalid = computed(() => invalid ?? formField.isInvalid.value);
const wiring = useFieldWiring({
  hasDescription: toRef(() => hasDescription),
  hasErrorMessage: toRef(() => hasErrorMessage),
  id: toRef(() => id),
  invalid: isInvalid,
});
const slotState = computed<FieldRootSlotState>(() => ({
  descriptionProps: wiring.descriptionProps.value,
  errorMessage: formField.errorMessage.value,
  errorMessageProps: wiring.errorMessageProps.value,
  errors: formField.errors.value,
  fieldProps: wiring.fieldProps.value,
  id: wiring.fieldId.value,
  invalid: isInvalid.value,
  labelProps: wiring.labelProps.value,
  name: formField.name.value,
}));

const context = fieldContext.provide({
  descriptionProps: wiring.descriptionProps,
  errorMessage: formField.errorMessage,
  errorMessageProps: wiring.errorMessageProps,
  errors: formField.errors,
  fieldProps: wiring.fieldProps,
  id: wiring.fieldId,
  invalid: isInvalid,
  labelProps: wiring.labelProps,
  name: formField.name,
});

watch(isInvalid, (next, previous) => {
  if (next !== previous) emit("invalid-change", next, formField.errors.value);
});

type FieldRootSetupExpose = Omit<
  FieldRootExpose,
  | "descriptionProps"
  | "element"
  | "errorMessage"
  | "errorMessageProps"
  | "errors"
  | "fieldProps"
  | "id"
  | "invalid"
  | "labelProps"
  | "name"
> & {
  readonly descriptionProps: ComputedRef<FieldTextProps>;
  readonly element: typeof element;
  readonly errorMessage: ComputedRef<string | undefined>;
  readonly errorMessageProps: ComputedRef<FieldTextProps>;
  readonly errors: ComputedRef<readonly FormFieldError[]>;
  readonly fieldProps: ComputedRef<FieldControlProps>;
  readonly id: ComputedRef<string>;
  readonly invalid: ComputedRef<boolean>;
  readonly labelProps: ComputedRef<FieldLabelProps>;
  readonly name: ComputedRef<string>;
};

const exposed = {
  descriptionProps: context.descriptionProps,
  element,
  errorMessage: context.errorMessage,
  errorMessageProps: context.errorMessageProps,
  errors: context.errors,
  fieldProps: context.fieldProps,
  id: context.id,
  invalid: context.invalid,
  labelProps: context.labelProps,
  name: context.name,
} satisfies FieldRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    data-vize-ui="field"
    part="root"
    :data-state="isInvalid ? 'invalid' : 'valid'"
    :data-invalid="isInvalid ? 'true' : 'false'"
    :data-name="formField.name.value"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
