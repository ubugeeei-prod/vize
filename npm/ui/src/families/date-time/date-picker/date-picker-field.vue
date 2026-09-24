<script setup lang="ts">
import { useTemplateRef } from "vue";

import DateField from "../date-field/date-field.vue";
import type {
  DateFieldExpose,
  DateFieldSlotState,
  FieldSegmentPlaceholders,
  FieldSegmentState,
} from "../date-field/date-field-types.ts";
import type { PlainDate } from "../calendar/plain-date.ts";
import { datePickerContext } from "./date-picker-context.ts";

const {
  placeholderValue = undefined,
  placeholders = undefined,
  emptyText = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  ariaInvalid = false,
} = defineProps<{
  /** Date whose segments seed arrow-key stepping from empty; defaults to the picker `today` or host date. @default undefined */
  readonly placeholderValue?: PlainDate | null;
  /** Placeholder text for empty segments. @default { year: "yyyy", month: "mm", day: "dd" } */
  readonly placeholders?: FieldSegmentPlaceholders;
  /** Text announced for empty segments. @default "Empty" */
  readonly emptyText?: string;
  /** Accessible name for the segment group. @default undefined */
  readonly ariaLabel?: string;
  /** Ids that label the segment group. @default undefined */
  readonly ariaLabelledby?: string;
  /** Ids that describe the group and every segment. @default undefined */
  readonly ariaDescribedby?: string;
  /** Id of the validation message used while invalid. @default undefined */
  readonly ariaErrormessage?: string;
  /** Force the invalid state. @default false */
  readonly ariaInvalid?: boolean;
}>();

defineSlots<{
  /** Extra content after the segments, typically DatePickerTrigger. Receives field state. */
  default(props: DateFieldSlotState): unknown;
  /** Segment content. Receives the segment state; defaults to its text. */
  segment(props: FieldSegmentState): unknown;
}>();

const context = datePickerContext.use();
const field = useTemplateRef<DateFieldExpose>("field");

function onChange(value: PlainDate | null, _previous: PlainDate | null, event: Event | null): void {
  context.setValue(value, event);
}

defineExpose({
  focus: (options?: FocusOptions) => field.value?.focus(undefined, options) ?? false,
});
</script>

<template>
  <DateField
    :id="context.fieldId.value"
    ref="field"
    :name="context.name.value"
    :model-value="context.value.value"
    :min="context.options.value.min"
    :max="context.options.value.max"
    :is-date-unavailable="context.options.value.isDateUnavailable"
    :placeholder-value="placeholderValue ?? context.options.value.today"
    :time-zone="context.options.value.timeZone"
    :locale="context.options.value.locale"
    :dir="context.options.value.dir"
    :disabled="context.disabled.value"
    :read-only="context.readOnly.value"
    :required="context.required.value"
    :placeholders
    :empty-text
    :aria-label
    :aria-labelledby
    :aria-describedby
    :aria-errormessage
    :aria-invalid
    data-picker-part="field"
    @change="onChange"
  >
    <template #segment="segment: FieldSegmentState"
      ><slot name="segment" v-bind="segment">{{ segment.text }}</slot></template
    >
    <template #default="state: DateFieldSlotState"><slot v-bind="state" /></template>
  </DateField>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
