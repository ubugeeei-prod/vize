<script setup lang="ts">
import { useTemplateRef } from "vue";

import type { DateMatcher, PlainDate } from "../calendar/plain-date.ts";
import { useDateField } from "./date-field-runtime.ts";
import type {
  DateFieldSlotState,
  FieldSegmentPlaceholders,
  FieldSegmentState,
} from "./date-field-types.ts";
import type { EditableSegmentType } from "./field-segments.ts";

const props = defineProps<{
  /** Consumer-owned group id; nullish values use a deterministic fallback. @default undefined */
  readonly id?: string | null | undefined;
  /** Hidden input name that submits the ISO date (`YYYY-MM-DD`) with forms. @default undefined */
  readonly name?: string | undefined;
  /** Controlled date; `undefined` selects uncontrolled mode and `null` clears. @default undefined */
  readonly modelValue?: PlainDate | null | undefined;
  /** Initial uncontrolled date and native form-reset target. @default null */
  readonly defaultValue?: PlainDate | null | undefined;
  /** Earliest valid date; later values mark the field invalid. @default undefined */
  readonly min?: PlainDate | null | undefined;
  /** Latest valid date; earlier values mark the field invalid. @default undefined */
  readonly max?: PlainDate | null | undefined;
  /** Predicate for dates that are entered but invalid. @default undefined */
  readonly isDateUnavailable?: DateMatcher | undefined;
  /** Date whose segments seed arrow-key stepping from empty; defaults to the host date at key time. @default undefined */
  readonly placeholderValue?: PlainDate | null | undefined;
  /** IANA time zone for the key-time host date fallback. @default undefined */
  readonly timeZone?: string | undefined;
  /** BCP 47 locale deciding segment order and separators; defaults to the nearest LocaleProvider. @default undefined */
  readonly locale?: string | undefined;
  /** Text direction for arrow-key segment movement; defaults to the nearest LocaleProvider. @default undefined */
  readonly dir?: "ltr" | "rtl" | undefined;
  /** Remove every segment from focus and editing. @default false */
  readonly disabled?: boolean | undefined;
  /** Keep segments focusable while blocking edits. @default false */
  readonly readOnly?: boolean | undefined;
  /** Mark the field as required for assistive technology. @default false */
  readonly required?: boolean | undefined;
  /** Placeholder text for empty segments. @default { year: "yyyy", month: "mm", day: "dd" } */
  readonly placeholders?: FieldSegmentPlaceholders | undefined;
  /** Text announced for empty segments. @default "Empty" */
  readonly emptyText?: string | undefined;
  /** Accessible name for the segment group. @default undefined */
  readonly ariaLabel?: string | undefined;
  /** Ids that label the segment group. @default undefined */
  readonly ariaLabelledby?: string | undefined;
  /** Ids that describe the group and every segment. @default undefined */
  readonly ariaDescribedby?: string | undefined;
  /** Id of the validation message used while invalid. @default undefined */
  readonly ariaErrormessage?: string | undefined;
  /** Force the invalid state in addition to built-in min/max/availability validation. @default false */
  readonly ariaInvalid?: boolean | undefined;
}>();

const emit = defineEmits<{
  /** Fired when segments request a new controlled date or `null` while incomplete. */
  "update:modelValue": [value: PlainDate | null];
  /** Fired after a distinct committed date with the previous date and triggering event. */
  change: [value: PlainDate | null, previous: PlainDate | null, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Extra content after the segments, such as a picker trigger. Receives field state. */
  default(props: DateFieldSlotState): unknown;
  /** Segment content. Receives the segment state; defaults to its text. */
  segment(props: FieldSegmentState): unknown;
}>();

const root = useTemplateRef<HTMLDivElement>("root");
const { field, id, direction, invalid, isoValue, slotState, segmentId, exposed } = useDateField(
  props,
  emit,
  root,
);

function editableType(segment: FieldSegmentState): EditableSegmentType {
  return segment.type === "literal" ? "year" : segment.type;
}

function onSegmentKeydown(segment: FieldSegmentState, event: KeyboardEvent): void {
  field.onSegmentKeydown(editableType(segment), event);
}

function onSegmentBeforeInput(segment: FieldSegmentState, event: InputEvent): void {
  field.onSegmentBeforeInput(editableType(segment), event);
}

function onSegmentFocus(segment: FieldSegmentState): void {
  field.onSegmentFocus(editableType(segment));
}

function onSegmentBlur(): void {
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
    data-vize-ui="date-field"
    :data-state="slotState.state"
    :data-value="isoValue || undefined"
    :data-dir="direction"
    :data-invalid="invalid ? 'true' : undefined"
    :data-disabled="slotState.disabled ? 'true' : undefined"
    :data-readonly="slotState.readOnly ? 'true' : undefined"
    :data-required="slotState.required ? 'true' : undefined"
  >
    <template v-for="segment in slotState.segments" :key="segment.index">
      <span
        v-if="segment.editable"
        :id="segmentId(segment.type)"
        role="spinbutton"
        tabindex="0"
        :contenteditable="slotState.disabled || slotState.readOnly ? undefined : 'true'"
        :inputmode="segment.type === 'dayPeriod' ? 'text' : 'numeric'"
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
        :aria-invalid="invalid ? 'true' : undefined"
        :aria-errormessage="invalid ? props.ariaErrormessage : undefined"
        data-vize-ui="date-field-segment"
        part="segment"
        :data-segment="segment.type"
        :data-placeholder="segment.placeholder ? 'true' : undefined"
        :data-invalid="invalid ? 'true' : undefined"
        @keydown="(event) => onSegmentKeydown(segment, event)"
        @beforeinput="(event) => onSegmentBeforeInput(segment, event)"
        @focus="() => onSegmentFocus(segment)"
        @blur="onSegmentBlur"
        ><slot name="segment" v-bind="segment">{{ segment.text }}</slot></span
      >
      <span v-else aria-hidden="true" data-vize-ui="date-field-literal" part="literal">{{
        segment.text
      }}</span>
    </template>
    <input
      v-if="props.name"
      type="hidden"
      :name="props.name"
      :value="isoValue"
      :disabled="slotState.disabled"
      data-vize-ui="date-field-input"
    />
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
