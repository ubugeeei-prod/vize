<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { resolveLocale, useLocaleValue } from "../../i18n/locale/locale-runtime.ts";
import Listbox from "../../selection/listbox/listbox.vue";
import ListboxItem from "../../selection/listbox/listbox-item.vue";
import type { ListboxExpose, ListboxProps } from "../../selection/listbox/listbox-types.ts";
import type { HourCycle, PlainTime } from "../time-field/plain-time.ts";
import { useTimePicker } from "./time-picker-runtime.ts";
import type { TimePickerExpose, TimePickerSlot, TimePickerSlotState } from "./time-picker-types.ts";

const props = defineProps<{
  /** Consumer-owned base id; the listbox uses `<id>-listbox`. Nullish values use a deterministic fallback. @default undefined */
  readonly id?: string | null | undefined;
  /** Controlled time; `undefined` selects uncontrolled mode and `null` clears. @default undefined */
  readonly modelValue?: PlainTime | null | undefined;
  /** Initial uncontrolled time. @default null */
  readonly defaultValue?: PlainTime | null | undefined;
  /** First slot, inclusive. @default { hour: 0, minute: 0, second: 0 } */
  readonly min?: PlainTime | null | undefined;
  /** Last possible slot, inclusive. @default { hour: 23, minute: 59, second: 59 } */
  readonly max?: PlainTime | null | undefined;
  /** Minutes between slots. @default 30 */
  readonly step?: number | undefined;
  /** `12` or `24`-hour labels; defaults to the locale clock. @default undefined */
  readonly hourCycle?: HourCycle | undefined;
  /** Predicate for slots rendered as disabled options. @default undefined */
  readonly isTimeUnavailable?: ((time: PlainTime) => boolean) | undefined;
  /** BCP 47 locale for labels and typeahead; defaults to the nearest LocaleProvider. @default undefined */
  readonly locale?: string | undefined;
  /** Disable the listbox. @default false */
  readonly disabled?: boolean | undefined;
  /** Keep navigation while ignoring selection requests. @default false */
  readonly readOnly?: boolean | undefined;
  /** Mark the listbox as required. @default false */
  readonly required?: boolean | undefined;
  /** Hidden input name that submits `HH:MM`. @default undefined */
  readonly name?: string | undefined;
  /** Accessible name for the listbox. @default undefined */
  readonly ariaLabel?: string | undefined;
  /** Ids that label the listbox. @default undefined */
  readonly ariaLabelledby?: string | undefined;
  /** Ids that describe the listbox. @default undefined */
  readonly ariaDescribedby?: string | undefined;
}>();

const emit = defineEmits<{
  /** Fired when the picker requests a new controlled time. */
  "update:modelValue": [value: PlainTime | null];
  /** Fired after a distinct selection with the previous time and triggering event. */
  change: [value: PlainTime | null, previous: PlainTime | null, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Option content. Receives the slot; defaults to its localized label. */
  option(props: TimePickerSlot): unknown;
}>();

const baseId = useDeterministicId({ id: () => props.id, hint: "time-picker" });
const localeValue = useLocaleValue();
const locale = computed(() => resolveLocale(props.locale ?? localeValue.value.locale));
const listbox = useTemplateRef<ListboxExpose>("listbox");
const picker = useTimePicker({
  value: () => props.modelValue,
  defaultValue: () => props.defaultValue,
  min: () => props.min,
  max: () => props.max,
  step: () => props.step,
  hourCycle: () => props.hourCycle,
  locale: () => locale.value,
  isTimeUnavailable: () => props.isTimeUnavailable,
  readOnly: () => props.readOnly === true,
  onUpdate: (value) => emit("update:modelValue", value),
  onChange: (value, previous, event) => emit("change", value, previous, event),
});
const listboxProps = computed<ListboxProps>(() => ({
  modelValue: picker.listboxValue.value,
  selectionMode: "single",
  disabled: props.disabled === true,
  required: props.required === true,
  id: deriveDeterministicId(baseId.value, "listbox"),
  ...(props.ariaLabel === undefined ? {} : { ariaLabel: props.ariaLabel }),
  ...(props.ariaLabelledby === undefined ? {} : { ariaLabelledby: props.ariaLabelledby }),
  ...(props.ariaDescribedby === undefined ? {} : { ariaDescribedby: props.ariaDescribedby }),
}));
const slotState = computed<TimePickerSlotState>(() => ({
  value: picker.value.value,
  slots: picker.slots.value,
  hourCycle: picker.hourCycle.value,
  disabled: props.disabled === true,
  readOnly: props.readOnly === true,
}));

function onChange(
  value: string | readonly string[] | null,
  _previous: unknown,
  event: Event | null,
): void {
  picker.onListboxChange(value, event);
}

defineExpose({
  value: picker.value,
  slots: picker.slots,
  hourCycle: picker.hourCycle,
  disabled: computed(() => props.disabled === true),
  readOnly: computed(() => props.readOnly === true),
  focus: (options?: FocusOptions) => listbox.value?.focus(options),
  setValue: (value: PlainTime | null) => picker.setValue(value),
} satisfies { readonly [Key in keyof TimePickerExpose]: unknown });
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="time-picker"
    part="root"
    :data-hour-cycle="slotState.hourCycle"
    :data-value="picker.listboxValue.value ?? undefined"
    :data-disabled="slotState.disabled ? 'true' : undefined"
    :data-readonly="slotState.readOnly ? 'true' : undefined"
  >
    <Listbox ref="listbox" v-bind="listboxProps" @change="onChange">
      <ListboxItem
        v-for="slot in slotState.slots"
        :key="slot.value"
        :value="slot.value"
        :text-value="slot.label"
        :disabled="slot.disabled"
        :data-time="slot.value"
      >
        <slot name="option" v-bind="slot">{{ slot.label }}</slot>
      </ListboxItem>
    </Listbox>
    <input
      v-if="props.name"
      type="hidden"
      :name="props.name"
      :value="picker.listboxValue.value ?? ''"
      :disabled="slotState.disabled"
      data-vize-ui="time-picker-input"
    />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
