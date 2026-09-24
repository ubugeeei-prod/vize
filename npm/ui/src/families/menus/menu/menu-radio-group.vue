<script setup lang="ts" generic="Value">
import { computed, shallowRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { menuGroupContext, menuRadioGroupContext } from "./menu-context.ts";
import type { MenuRadioGroupExpose, MenuRadioGroupSlotState } from "./menu-types.ts";

const {
  modelValue = undefined,
  defaultValue = null,
  disabled = false,
  equals = Object.is,
} = defineProps<{
  /**
   * Controlled value (`v-model`). `undefined` selects uncontrolled behavior; `null` clears.
   *
   * @default undefined
   */
  readonly modelValue?: Value | null;

  /**
   * Initial value for uncontrolled use.
   *
   * @default null
   */
  readonly defaultValue?: Value | null;

  /**
   * Block activation of every radio item in the group.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Equality used to decide which item is checked (for example, compare object ids).
   *
   * @default Object.is
   */
  readonly equals?: (left: Value, right: Value) => boolean;
}>();

const emit = defineEmits<{
  /** Fired with the newly selected value (supports `v-model`). */
  "update:modelValue": [value: Value | null];
  /** Fired after a distinct value is selected, with the previous value and native event. */
  "value-change": [value: Value | null, previous: Value | null, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Radio items. Receives the current value. */
  default?(props: MenuRadioGroupSlotState<Value>): unknown;
}>();

const valueState = useControllableState<Value | null>({
  value: () => modelValue,
  defaultValue: () => defaultValue,
  equals: (left, right) => (left === null || right === null ? left === right : equals(left, right)),
});
const labelId = shallowRef<string | null>(null);
const groupId = useDeterministicId({ hint: "menu-radio-group" });
const disabledState = computed(() => disabled);

function setValue(next: Value | null, event: Event | null = null): boolean {
  const previous = valueState.value.value;
  if (!valueState.set(next)) return false;
  emit("update:modelValue", next);
  emit("value-change", next, previous, event);
  return true;
}

function isChecked(candidate: unknown): boolean {
  const current = valueState.value.value;
  return current !== null && candidate !== null && sameValue(current, candidate);
}

function sameValue(current: Value, candidate: unknown): boolean {
  return Object.is(current, candidate) || equals(current, accept(candidate));
}

// Vue injection erases the item's type parameter. `MenuRadioItem<Value>` is
// declared with the same `Value` at the call site, so the value an item hands
// back is the value the consumer bound to this group.
function accept(candidate: unknown): Value {
  return candidate as Value;
}

menuRadioGroupContext.provide({
  disabled: disabledState,
  isChecked,
  select: (candidate, event) => setValue(candidate === null ? null : accept(candidate), event),
});
menuGroupContext.provide({ labelId });

defineExpose({
  setValue: (value: Value | null) => setValue(value),
  value: valueState.value,
} satisfies Omit<MenuRadioGroupExpose<Value>, "value"> & {
  readonly value: typeof valueState.value;
});
</script>

<template>
  <div
    :id="groupId"
    role="group"
    :aria-labelledby="labelId ?? undefined"
    data-vize-ui="menu-radio-group"
    part="radio-group"
    :data-disabled="disabledState ? 'true' : undefined"
  >
    <slot :value="valueState.value.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
