<script setup lang="ts">
import { useTemplateRef } from "vue";

import type { DurationUnit, DurationValue } from "./duration.ts";
import { useDurationField } from "./duration-field-runtime.ts";
import {
  fieldValidationInputStyle,
  useFieldValidationInput,
} from "../date-field/field-segment-runtime.ts";
import type {
  DurationFieldSlotState,
  DurationSegmentState,
  DurationUnitDisplay,
} from "./duration-field-types.ts";

const props = defineProps<{
  /** Consumer-owned group id; nullish values use a deterministic fallback. @default undefined */
  readonly id?: string | null | undefined;
  /** Input name that submits the ISO 8601 duration (for example `PT1H30M`). @default undefined */
  readonly name?: string | undefined;
  /** Controlled duration; `undefined` selects uncontrolled mode and `null` clears. @default undefined */
  readonly modelValue?: DurationValue | null | undefined;
  /** Initial uncontrolled duration and native form-reset target. @default null */
  readonly defaultValue?: DurationValue | null | undefined;
  /** Units rendered as segments, always shown largest first. @default ["hours", "minutes"] */
  readonly fields?: readonly DurationUnit[] | undefined;
  /** Unit label width next to each segment. @default "short" */
  readonly unitDisplay?: DurationUnitDisplay | undefined;
  /** Upper bound of units that do not roll into a larger edited unit. @default 9999 */
  readonly maxValue?: number | undefined;
  /** BCP 47 locale for unit labels; defaults to the nearest LocaleProvider. @default undefined */
  readonly locale?: string | undefined;
  /** Text direction for arrow-key segment movement. @default undefined */
  readonly dir?: "ltr" | "rtl" | undefined;
  /** Disable every segment. @default false */
  readonly disabled?: boolean | undefined;
  /** Keep segments focusable while blocking edits. @default false */
  readonly readOnly?: boolean | undefined;
  /** Require a complete value; participates in native form validation. @default false */
  readonly required?: boolean | undefined;
  /** Placeholder shown in empty segments. @default "––" */
  readonly placeholder?: string | undefined;
  /** Text announced for empty segments. @default "Empty" */
  readonly emptyText?: string | undefined;
  /** Accessible name for the segment group. @default undefined */
  readonly ariaLabel?: string | undefined;
  /** Ids that label the segment group. @default undefined */
  readonly ariaLabelledby?: string | undefined;
  /** Ids that describe the group and every segment. @default undefined */
  readonly ariaDescribedby?: string | undefined;
}>();

const emit = defineEmits<{
  /** Fired when segments request a new controlled duration or `null` while incomplete. */
  "update:modelValue": [value: DurationValue | null];
  /** Fired after a distinct committed duration with the previous value and triggering event. */
  change: [value: DurationValue | null, previous: DurationValue | null, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Extra content after the segments. Receives field state. */
  default(props: DurationFieldSlotState): unknown;
  /** Segment amount content. Receives the segment state; defaults to its text. */
  segment(props: DurationSegmentState): unknown;
}>();

const root = useTemplateRef<HTMLDivElement>("root");
const input = useTemplateRef<HTMLInputElement>("input");
const field = useDurationField(props, emit, root);
const { id, direction, isoValue, slotState, exposed, segmentId } = field;
const validation = useFieldValidationInput({
  input,
  invalid: () => false,
  message: () => "",
  focus: () => field.focus(),
});

function onKeydown(segment: DurationSegmentState, event: KeyboardEvent): void {
  field.onSegmentKeydown(segment.unit, event);
}

function onBeforeInput(segment: DurationSegmentState, event: InputEvent): void {
  field.onSegmentBeforeInput(segment.unit, event);
}

function onFocus(segment: DurationSegmentState): void {
  field.onSegmentFocus(segment.unit);
}

function onBlur(): void {
  field.onSegmentBlur();
}

defineExpose(exposed);
</script>

<template>
  <div
    :id="id"
    ref="root"
    part="root"
    role="group"
    :dir="direction"
    :aria-label="props.ariaLabel"
    :aria-labelledby="props.ariaLabelledby"
    :aria-describedby="props.ariaDescribedby"
    :aria-disabled="slotState.disabled ? 'true' : undefined"
    data-vize-ui="duration-field"
    :data-state="slotState.state"
    :data-value="isoValue || undefined"
    :data-disabled="slotState.disabled ? 'true' : undefined"
    :data-readonly="slotState.readOnly ? 'true' : undefined"
    :data-required="slotState.required ? 'true' : undefined"
  >
    <span
      v-for="segment in slotState.segments"
      :key="segment.unit"
      data-vize-ui="duration-field-unit"
      part="unit"
      :data-unit="segment.unit"
    >
      <span
        v-if="segment.prefix"
        aria-hidden="true"
        data-vize-ui="duration-field-affix"
        part="affix"
        >{{ segment.prefix }}</span
      >
      <span
        :id="segmentId(segment.unit)"
        role="spinbutton"
        tabindex="0"
        :contenteditable="slotState.disabled || slotState.readOnly ? undefined : 'true'"
        inputmode="numeric"
        spellcheck="false"
        autocapitalize="off"
        enterkeyhint="next"
        :aria-label="segment.label"
        :aria-describedby="props.ariaDescribedby"
        :aria-valuenow="segment.value ?? undefined"
        :aria-valuemin="segment.min"
        :aria-valuemax="segment.max"
        :aria-valuetext="segment.valueText"
        :aria-disabled="slotState.disabled ? 'true' : undefined"
        :aria-readonly="slotState.readOnly ? 'true' : undefined"
        :aria-required="slotState.required ? 'true' : undefined"
        data-vize-ui="duration-field-segment"
        part="segment"
        :data-unit="segment.unit"
        :data-placeholder="segment.placeholder ? 'true' : undefined"
        @keydown="(event) => onKeydown(segment, event)"
        @beforeinput="(event) => onBeforeInput(segment, event)"
        @focus="() => onFocus(segment)"
        @blur="onBlur"
        ><slot name="segment" v-bind="segment">{{ segment.text }}</slot></span
      >
      <span
        v-if="segment.suffix"
        aria-hidden="true"
        data-vize-ui="duration-field-affix"
        part="affix"
        >{{ segment.suffix }}</span
      >
    </span>
    <input
      v-if="props.name || slotState.required"
      ref="input"
      type="text"
      :name="props.name"
      :value="isoValue"
      :required="slotState.required"
      :disabled="slotState.disabled"
      tabindex="-1"
      aria-hidden="true"
      :aria-labelledby="id"
      autocomplete="off"
      data-vize-ui="duration-field-input"
      :style="fieldValidationInputStyle"
      @invalid="validation.onInvalid"
    />
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
