<script setup lang="ts">
import { useTemplateRef } from "vue";

import { useRating } from "./rating-runtime.ts";
import type {
  RatingAriaInvalid,
  RatingDirection,
  RatingItemSlotState,
  RatingSlotState,
  RatingValue,
} from "./rating-types.ts";

const props = defineProps<{
  /** Consumer-owned group id; nullish values use a deterministic fallback. @default undefined */
  readonly id?: string | null;
  /** Native radio field name submitted with the selected rating. @default undefined */
  readonly name?: string;
  /** Controlled rating value; undefined selects uncontrolled behavior and null clears. @default undefined */
  readonly modelValue?: RatingValue;
  /** Initial uncontrolled rating and native form-reset target. @default null */
  readonly defaultValue?: RatingValue;
  /** Lowest generated rating value. @default 1 */
  readonly min?: number;
  /** Highest generated rating value; when omitted, count derives it. @default undefined */
  readonly max?: number;
  /** Number of generated rating choices when max is omitted. @default 5 */
  readonly count?: number;
  /** Allow current-item activation, Escape, Delete, or Backspace to clear. @default false */
  readonly clearable?: boolean;
  /** Disable native focus, activation, and form submission. @default false */
  readonly disabled?: boolean;
  /** Keep focus and current submission while preventing user changes. @default false */
  readonly readOnly?: boolean;
  /** Mark the generated native radio set as required. @default false */
  readonly required?: boolean;
  /** Text direction for horizontal arrow keys and data attributes. @default "ltr" */
  readonly dir?: RatingDirection;
  /** Prefix for each generated radio accessible name. @default "Rating" */
  readonly itemLabel?: string;
  /** Accessible name when no visible label or aria-labelledby supplies one. @default undefined */
  readonly ariaLabel?: string;
  /** Space-separated ids that label the rating group. @default undefined */
  readonly ariaLabelledby?: string;
  /** Space-separated ids that describe the rating group. @default undefined */
  readonly ariaDescribedby?: string;
  /** Id of the validation error message used while invalid. @default undefined */
  readonly ariaErrormessage?: string;
  /** Invalid state announced to assistive technology. @default false */
  readonly ariaInvalid?: RatingAriaInvalid;
}>();

const emit = defineEmits<{
  /** Fired when the rating requests a new controlled value. */
  "update:modelValue": [value: RatingValue];
  /** Fired after user activation requests a distinct rating value. */
  change: [value: RatingValue, previous: RatingValue, nativeEvent: Event];
  /** Fired after user activation clears a previously selected rating. */
  clear: [previous: number, nativeEvent: Event];
}>();

defineSlots<{
  /** Renders optional summary or output with the normalized Rating state. */
  default(props: RatingSlotState): unknown;
  /** Renders each generated rating indicator with item and group state. */
  item(props: RatingItemSlotState): unknown;
}>();

const root = useTemplateRef<HTMLSpanElement>("root");
const {
  ariaInvalidValue,
  clearableState,
  controlId,
  currentValue,
  dataState,
  directionState,
  disabledState,
  exposed,
  invalidState,
  itemAriaLabel,
  itemCount,
  itemId,
  itemStates,
  items,
  maxValue,
  minValue,
  onItemChange,
  onItemClick,
  onItemKeydown,
  percent,
  readOnlyState,
  requiredState,
  intrinsicProps,
} = useRating(props, emit, root);

defineExpose(exposed);
</script>

<template>
  <span
    :id="controlId"
    ref="root"
    role="radiogroup"
    :dir="directionState"
    :aria-label="props.ariaLabel"
    :aria-labelledby="props.ariaLabelledby"
    :aria-describedby="props.ariaDescribedby"
    :aria-errormessage="ariaInvalidValue === undefined ? undefined : props.ariaErrormessage"
    :aria-invalid="ariaInvalidValue"
    :aria-disabled="disabledState ? 'true' : undefined"
    :aria-readonly="readOnlyState ? 'true' : undefined"
    :aria-required="requiredState ? 'true' : undefined"
    data-vize-ui="rating"
    part="root"
    :data-state="dataState"
    :data-value="currentValue ?? undefined"
    :data-min="minValue"
    :data-max="maxValue"
    :data-count="itemCount"
    :data-percent="percent"
    :data-dir="directionState"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-readonly="readOnlyState ? 'true' : undefined"
    :data-required="requiredState ? 'true' : undefined"
    :data-invalid="invalidState ? 'true' : undefined"
    :data-clearable="clearableState ? 'true' : undefined"
    v-bind="intrinsicProps"
  >
    <label
      v-for="item in itemStates as readonly RatingItemSlotState[]"
      :key="item.value"
      data-vize-ui="rating-item"
      part="item"
      :data-state="item.state"
      :data-value="item.value"
      :data-index="item.index"
      :data-active="item.active ? 'true' : 'false'"
      :data-checked="item.checked ? 'true' : 'false'"
      :data-disabled="item.disabled ? 'true' : undefined"
      :data-readonly="item.readOnly ? 'true' : undefined"
      :data-required="item.required ? 'true' : undefined"
      :data-invalid="item.invalid ? 'true' : undefined"
      :data-dir="item.direction"
    >
      <input
        :id="itemId(item.value)"
        type="radio"
        :name="props.name"
        :value="String(item.value)"
        :checked="item.checked"
        :disabled="item.disabled"
        :required="item.required"
        :tabindex="item.checked || (currentValue === null && item.index === 0) ? 0 : -1"
        :aria-label="itemAriaLabel(item.value)"
        :aria-invalid="ariaInvalidValue"
        :aria-readonly="item.readOnly ? 'true' : undefined"
        data-vize-ui="rating-control"
        part="control"
        :data-state="item.state"
        :data-value="item.value"
        :data-index="item.index"
        :data-active="item.active ? 'true' : 'false'"
        :data-checked="item.checked ? 'true' : 'false'"
        :data-disabled="item.disabled ? 'true' : undefined"
        :data-readonly="item.readOnly ? 'true' : undefined"
        :data-required="item.required ? 'true' : undefined"
        :data-invalid="item.invalid ? 'true' : undefined"
        :data-dir="item.direction"
        @click="(event) => onItemClick(event, item.value)"
        @change="(event) => onItemChange(event, item.value)"
        @keydown="(event) => onItemKeydown(event, item.value)"
      />
      <span
        aria-hidden="true"
        data-vize-ui="rating-indicator"
        part="indicator"
        :data-state="item.state"
        :data-value="item.value"
        :data-index="item.index"
        :data-active="item.active ? 'true' : 'false'"
        :data-checked="item.checked ? 'true' : 'false'"
        :data-disabled="item.disabled ? 'true' : undefined"
        :data-readonly="item.readOnly ? 'true' : undefined"
        :data-invalid="item.invalid ? 'true' : undefined"
        :data-dir="item.direction"
      >
        <slot name="item" v-bind="item">{{ item.value }}</slot>
      </span>
    </label>
    <slot
      :value="currentValue"
      :min="minValue"
      :max="maxValue"
      :count="itemCount"
      :items="items"
      :percent="percent"
      :direction="directionState"
      :disabled="disabledState"
      :read-only="readOnlyState"
      :required="requiredState"
      :invalid="invalidState"
      :clearable="clearableState"
      :state="dataState"
    />
  </span>
</template>

<style scoped>
/* Headless by design. Native radio styling remains entirely consumer-owned. */
</style>
