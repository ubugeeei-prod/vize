<script setup lang="ts">
import { computed } from "vue";
import type { ComputedRef } from "vue";

import PopoverRoot from "../../overlays/popover/popover-root.vue";
import { deriveDeterministicId } from "../../foundations/id/deterministic-id.ts";
import type { DateTimeNow } from "../calendar/calendar-today.ts";
import { formatIsoDate } from "../calendar/plain-date.ts";
import type { DateMatcher, PlainDate } from "../calendar/plain-date.ts";
import { datePickerContext } from "./date-picker-context.ts";
import { usePickerRoot, usePickerValue } from "./date-picker-runtime.ts";
import type { DatePickerRootExpose, DatePickerSlotState } from "./date-picker-types.ts";

const {
  id = undefined,
  modelValue = undefined,
  defaultValue = undefined,
  open = undefined,
  defaultOpen = false,
  modal = false,
  min = undefined,
  max = undefined,
  isDateUnavailable = undefined,
  locale = undefined,
  dir = undefined,
  today = undefined,
  now = undefined,
  timeZone = undefined,
  disabled = false,
  readOnly = false,
  required = false,
  name = undefined,
  closeOnSelect = true,
} = defineProps<{
  /** Consumer-owned base id; nullish values use a deterministic fallback. @default undefined */
  readonly id?: string | null;
  /** Controlled date; `undefined` selects uncontrolled mode and `null` clears. @default undefined */
  readonly modelValue?: PlainDate | null;
  /** Initial uncontrolled date. @default null */
  readonly defaultValue?: PlainDate | null;
  /** Controlled popover open state; `undefined` selects uncontrolled mode. @default undefined */
  readonly open?: boolean;
  /** Initial uncontrolled open state. @default false */
  readonly defaultOpen?: boolean;
  /** Make outside content inert and contain focus while open. @default false */
  readonly modal?: boolean;
  /** Earliest selectable date, inclusive. @default undefined */
  readonly min?: PlainDate | null;
  /** Latest selectable date, inclusive. @default undefined */
  readonly max?: PlainDate | null;
  /** Predicate for dates that cannot be selected. @default undefined */
  readonly isDateUnavailable?: DateMatcher;
  /** BCP 47 locale for the field and calendar; defaults to the nearest LocaleProvider. @default undefined */
  readonly locale?: string;
  /** Text direction; defaults to the nearest LocaleProvider. @default undefined */
  readonly dir?: "ltr" | "rtl";
  /** Explicit current date for the calendar; the SSR-safe way to mark today. @default undefined */
  readonly today?: PlainDate | null;
  /** Injectable clock evaluated during setup on server and client. @default undefined */
  readonly now?: DateTimeNow;
  /** IANA time zone used with `now` and host clocks. @default undefined */
  readonly timeZone?: string;
  /** Disable the field, trigger, and calendar. @default false */
  readonly disabled?: boolean;
  /** Lock the value while the calendar stays browsable. @default false */
  readonly readOnly?: boolean;
  /** Mark the field as required. @default false */
  readonly required?: boolean;
  /** Hidden input name that submits the ISO date with forms. @default undefined */
  readonly name?: string;
  /** Close the popover after a calendar selection. @default true */
  readonly closeOnSelect?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the picker requests a new controlled date. */
  "update:modelValue": [value: PlainDate | null];
  /** Fired after a distinct committed date with the previous date and triggering event. */
  change: [value: PlainDate | null, previous: PlainDate | null, nativeEvent: Event | null];
  /** Fired when the picker requests a new controlled open state. */
  "update:open": [value: boolean];
  /** Fired after any distinct open-state request. */
  "open-change": [value: boolean, previous: boolean, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Picker composition: DatePickerField, DatePickerTrigger, DatePickerContent, DatePickerCalendar. */
  default(props: DatePickerSlotState): unknown;
}>();

const picker = usePickerRoot({
  id: () => id,
  hint: "date-picker",
  open: () => open,
  defaultOpen: () => defaultOpen,
  disabled: () => disabled,
  readOnly: () => readOnly,
  required: () => required,
  options: () => ({
    min,
    max,
    isDateUnavailable,
    locale,
    dir,
    today,
    now,
    timeZone,
    closeOnSelect,
    allowNonContiguousRanges: false,
  }),
  onOpenChange: (value, previous, event) => {
    emit("update:open", value);
    emit("open-change", value, previous, event);
  },
});
const { baseId, popoverId, state, shared, onPopoverOpenChange } = picker;
const selection = usePickerValue({
  value: () => modelValue,
  defaultValue: () => defaultValue,
  locked: () => shared.disabled.value || shared.readOnly.value,
  onUpdate: (value) => emit("update:modelValue", value),
  onChange: (value, previous, event) => emit("change", value, previous, event),
});
const isoValue = computed(() =>
  selection.value.value ? formatIsoDate(selection.value.value) : undefined,
);

datePickerContext.provide({
  ...shared,
  fieldId: computed(() => deriveDeterministicId(baseId.value, "field")),
  name: computed(() => name),
  value: selection.value,
  setValue: selection.setValue,
});

const slotState = computed<DatePickerSlotState>(() => ({
  value: selection.value.value,
  open: shared.open.value,
  disabled: shared.disabled.value,
  readOnly: shared.readOnly.value,
  state: state.value,
}));

type DatePickerRootSetupExpose = {
  readonly [Key in keyof DatePickerRootExpose]: DatePickerRootExpose[Key] extends (
    ...args: never[]
  ) => unknown
    ? DatePickerRootExpose[Key]
    : ComputedRef<DatePickerRootExpose[Key]>;
};

const exposed = {
  value: selection.value,
  open: shared.open,
  disabled: shared.disabled,
  readOnly: shared.readOnly,
  state,
  setValue: (value: PlainDate | null) => selection.setValue(value),
  setOpen: (value: boolean) => shared.setOpen(value),
} satisfies DatePickerRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="date-picker"
    part="root"
    :data-state="state"
    :data-value="isoValue"
    :data-disabled="slotState.disabled ? 'true' : undefined"
    :data-readonly="slotState.readOnly ? 'true' : undefined"
  >
    <PopoverRoot
      :id="popoverId"
      :open="slotState.open"
      :modal
      :disabled="slotState.disabled"
      @open-change="onPopoverOpenChange"
    >
      <slot v-bind="slotState" />
    </PopoverRoot>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
