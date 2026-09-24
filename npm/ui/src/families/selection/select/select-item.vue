<script setup lang="ts" generic="T">
import { computed, onUnmounted, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { selectContext, selectItemContext } from "./select-context.ts";
import type { SelectItemExpose, SelectItemSlotState, SelectItemState } from "./select-types.ts";

const {
  id = undefined,
  value,
  disabled = false,
  textValue = undefined,
  index = undefined,
  ariaLabel = undefined,
} = defineProps<{
  /**
   * Consumer-owned option id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /** Option value compared with the root selection through `by`. @default required */
  readonly value: T;

  /**
   * Disable this option.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Explicit label for typeahead and `SelectValue`. `undefined` extracts the option text.
   *
   * @default undefined
   */
  readonly textValue?: string;

  /**
   * Absolute index inside the root `items`, required inside `SelectVirtualizer`.
   *
   * @default undefined
   */
  readonly index?: number;

  /**
   * Accessible name when the option text is not enough.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Option content. Receives selected, active, and disabled state. */
  default?(props: SelectItemSlotState<T>): unknown;
}>();

const context = selectContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const optionId = useDeterministicId({ id: () => id, hint: "select-option" });
const itemDisabled = computed(
  () => context.disabled.value || disabled || context.isValueDisabled(value),
);
const selected = computed(() => context.isSelected(value));
const visible = computed(() => context.isItemVisible(value, textValue ?? context.textOf(value)));
const active = computed(() => context.activeKey.value === optionId.value);
const state = computed<SelectItemState>(() => (selected.value ? "checked" : "unchecked"));
const slotState = computed<SelectItemSlotState<T>>(() => ({
  active: active.value,
  disabled: itemDisabled.value,
  selected: selected.value,
  state: state.value,
  value,
}));
const interactiveProps = computed<{
  readonly role: "option";
  readonly onClick: (event: MouseEvent) => void;
  readonly onPointermove: (event: PointerEvent) => void;
}>(() => ({
  role: "option",
  onClick,
  onPointermove,
}));

let registration: CollectionRegistration<string> | null = null;

// Filtered-out options (Combobox) stay mounted but leave the collection, so
// navigation, typeahead, and emptiness only see visible options.
function register(): void {
  registration?.unregister();
  registration = null;
  if (!visible.value) return;
  registration = context.registerItem({
    disabled: itemDisabled,
    element,
    id: optionId,
    index: () => index,
    textValue: () => textValue,
    value,
  });
}

watch([() => value, optionId, visible], register, { flush: "sync", immediate: true });
watch(
  [element, () => textValue],
  () => {
    const text = textValue ?? element.value?.textContent?.trim() ?? "";
    context.rememberText(value, text);
  },
  { flush: "post", immediate: true },
);
onUnmounted(() => {
  registration?.unregister();
  registration = null;
});

selectItemContext.provide({ active, disabled: itemDisabled, selected });

function onPointermove(event: PointerEvent): void {
  if (itemDisabled.value || active.value || event.pointerType === "touch") return;
  context.highlight(optionId.value);
}

function onClick(event: MouseEvent): void {
  if (itemDisabled.value) return;
  context.highlight(optionId.value);
  context.choose(value, event);
}

function select(): boolean {
  if (itemDisabled.value) return false;
  return context.choose(value, null);
}

type SelectItemSetupExpose = Omit<
  SelectItemExpose<T>,
  "active" | "element" | "selected" | "value"
> & {
  readonly active: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly selected: ComputedRef<boolean>;
  readonly value: T;
};

const exposed = { active, element, select, selected, value } satisfies SelectItemSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="optionId"
    ref="element"
    v-bind="interactiveProps"
    :aria-selected="selected ? 'true' : 'false'"
    :aria-disabled="itemDisabled ? 'true' : undefined"
    :aria-label="ariaLabel"
    :hidden="visible ? undefined : true"
    :data-vize-ui="`${context.partPrefix}-item`"
    part="item"
    :data-state="state"
    :data-highlighted="active ? 'true' : undefined"
    :data-disabled="itemDisabled ? 'true' : undefined"
    :data-index="index"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
