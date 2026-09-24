<script setup lang="ts">
import { computed } from "vue";
import type { ComputedRef } from "vue";

import PopoverRoot from "../../overlays/popover/popover-root.vue";
import { deriveDeterministicId } from "../../foundations/id/deterministic-id.ts";
import type { DateTimeNow } from "../calendar/calendar-today.ts";
import { formatIsoDate } from "../calendar/plain-date.ts";
import type { DateMatcher, DateRange, PlainDate } from "../calendar/plain-date.ts";
import { usePickerRoot } from "../date-picker/date-picker-runtime.ts";
import { dateRangePickerContext } from "./date-range-picker-context.ts";
import { useRangePickerValue } from "./date-range-picker-runtime.ts";
import type {
  DateRangePickerRootExpose,
  DateRangePickerSlotState,
} from "./date-range-picker-types.ts";

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
  startName = undefined,
  endName = undefined,
  allowNonContiguousRanges = false,
  closeOnSelect = true,
} = defineProps<{
  /** Consumer-owned base id; nullish values use a deterministic fallback. @default undefined */
  readonly id?: string | null;
  /** Controlled range; `undefined` selects uncontrolled mode and `null` clears. @default undefined */
  readonly modelValue?: DateRange | null;
  /** Initial uncontrolled range. @default null */
  readonly defaultValue?: DateRange | null;
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
  /** Hidden input name that submits the ISO start date with forms. @default undefined */
  readonly startName?: string;
  /** Hidden input name that submits the ISO end date with forms. @default undefined */
  readonly endName?: string;
  /** Allow committed ranges to span unavailable dates. @default false */
  readonly allowNonContiguousRanges?: boolean;
  /** Close the popover after a calendar selection. @default true */
  readonly closeOnSelect?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the picker requests a new controlled range. */
  "update:modelValue": [value: DateRange | null];
  /** Fired after a distinct committed range with the previous range and triggering event. */
  change: [value: DateRange | null, previous: DateRange | null, nativeEvent: Event | null];
  /** Fired when the picker requests a new controlled open state. */
  "update:open": [value: boolean];
  /** Fired after any distinct open-state request. */
  "open-change": [value: boolean, previous: boolean, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Picker composition: start/end DateRangePickerField, trigger, content, and DateRangePickerCalendar. */
  default(props: DateRangePickerSlotState): unknown;
}>();

const picker = usePickerRoot({
  id: () => id,
  hint: "date-range-picker",
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
    allowNonContiguousRanges,
  }),
  onOpenChange: (value, previous, event) => {
    emit("update:open", value);
    emit("open-change", value, previous, event);
  },
});
const { baseId, popoverId, state, shared, onPopoverOpenChange } = picker;
const range = useRangePickerValue({
  value: () => modelValue,
  defaultValue: () => defaultValue,
  locked: () => shared.disabled.value || shared.readOnly.value,
  onUpdate: (value) => emit("update:modelValue", value),
  onChange: (value, previous, event) => emit("change", value, previous, event),
});
const isoStart = computed(() =>
  range.value.value ? formatIsoDate(range.value.value.start) : undefined,
);
const isoEnd = computed(() =>
  range.value.value ? formatIsoDate(range.value.value.end) : undefined,
);

dateRangePickerContext.provide({
  ...shared,
  value: range.value,
  setValue: range.setValue,
  drafts: range.drafts,
  setBoundary: range.setBoundary,
  fieldIds: computed(() => ({
    start: deriveDeterministicId(baseId.value, "start"),
    end: deriveDeterministicId(baseId.value, "end"),
  })),
  names: computed(() => ({ start: startName, end: endName })),
});

const slotState = computed<DateRangePickerSlotState>(() => ({
  value: range.value.value,
  open: shared.open.value,
  disabled: shared.disabled.value,
  readOnly: shared.readOnly.value,
  state: state.value,
}));

type DateRangePickerRootSetupExpose = {
  readonly [Key in keyof DateRangePickerRootExpose]: DateRangePickerRootExpose[Key] extends (
    ...args: never[]
  ) => unknown
    ? DateRangePickerRootExpose[Key]
    : ComputedRef<DateRangePickerRootExpose[Key]>;
};

const exposed = {
  value: range.value,
  open: shared.open,
  disabled: shared.disabled,
  readOnly: shared.readOnly,
  state,
  setValue: (value: DateRange | null) => range.setValue(value),
  setOpen: (value: boolean) => shared.setOpen(value),
} satisfies DateRangePickerRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="date-range-picker"
    part="root"
    :data-state="state"
    :data-start="isoStart"
    :data-end="isoEnd"
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
