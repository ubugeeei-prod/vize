<script setup lang="ts">
import { computed, onUnmounted, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import type { CollectionRegistration } from "../../../collection.ts";
import { useDeterministicId } from "../../../deterministic-id.ts";
import { listboxContext } from "./listbox-context.ts";
import type { ListboxItemExpose, ListboxItemSlotState, ListboxItemState } from "./listbox-types.ts";

const {
  id = undefined,
  value,
  disabled = false,
  textValue = undefined,
  order = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Consumer-owned option id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /** Stable option value used for selection and collection identity. @default required */
  readonly value: string;

  /**
   * Disable this option while preserving the rest of the listbox.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Explicit typeahead text. `null` and `undefined` select accessible DOM text extraction.
   *
   * @default undefined
   */
  readonly textValue?: string | null;

  /**
   * Deterministic order for virtualized or server-only options.
   *
   * @default undefined
   */
  readonly order?: number;

  /**
   * Accessible name when no option text supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label this option.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe this option.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** Option content. Receives active, selected, disabled, and mode state. */
  default(props: ListboxItemSlotState): unknown;

  /** Optional selection indicator controlled by the consumer. */
  indicator(props: ListboxItemSlotState): unknown;
}>();

const context = listboxContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const optionRole = "option" as const;
const optionId = useDeterministicId({ id: () => id, hint: "listbox-option" });
const itemDisabled = computed(() => context.disabled.value || disabled);
const selected = computed(() => context.selectedValues.value.has(value));
const active = computed(() => context.activeValue.value === value && !itemDisabled.value);
const itemState = computed<ListboxItemState>(() => {
  if (itemDisabled.value) return "disabled";
  return selected.value ? "selected" : "unselected";
});
const itemSelectionMode = computed(() => context.selectionMode.value);
const slotState = computed<ListboxItemSlotState>(() => ({
  active: active.value,
  disabled: itemDisabled.value,
  selected: selected.value,
  selectionMode: itemSelectionMode.value,
  state: itemState.value,
  value,
}));
const optionDomId = computed(() => itemProps.value.id ?? optionId.value);
const itemInteractiveProps = computed<{
  readonly role: "option";
  readonly onPointerdown: (event: PointerEvent) => void;
  readonly onClick: (event: MouseEvent) => void;
}>(() => ({
  role: optionRole,
  onPointerdown,
  onClick,
}));
let registration: CollectionRegistration<string> | null = null;

function register(): void {
  registration?.unregister();
  registration = context.registerItem({
    disabled: itemDisabled,
    element,
    id: optionId,
    order: () => order,
    textValue: () => textValue,
    value,
  });
}

watch(() => value, register, { flush: "sync", immediate: true });
onUnmounted(() => {
  registration?.unregister();
  registration = null;
});

const itemProps = computed(() => context.getItemProps(value));

function onPointerdown(event: PointerEvent): void {
  if (itemDisabled.value) return;
  itemProps.value.onPointerdown(event);
  context.focus({ preventScroll: true });
}

function onClick(event: MouseEvent): void {
  if (itemDisabled.value) return;
  if (context.selectionMode.value === "multiple") context.toggleValue(value, event);
  else context.selectValue(value, event);
}

function focus(options?: FocusOptions): void {
  if (context.setActiveValue(value)) context.focus(options);
}

function select(): boolean {
  return context.selectValue(value, null);
}

type ListboxItemSetupExpose = Omit<
  ListboxItemExpose,
  "active" | "disabled" | "element" | "selected" | "selectionMode" | "state"
> & {
  readonly active: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly selected: ComputedRef<boolean>;
  readonly selectionMode: ComputedRef<ListboxItemSlotState["selectionMode"]>;
  readonly state: ComputedRef<ListboxItemState>;
};

const exposed = {
  active,
  disabled: itemDisabled,
  element,
  focus,
  select,
  selected,
  selectionMode: context.selectionMode,
  state: itemState,
  value,
} satisfies ListboxItemSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    v-bind="itemInteractiveProps"
    :id="optionDomId"
    ref="element"
    :aria-selected="selected ? 'true' : 'false'"
    :aria-disabled="itemDisabled ? 'true' : undefined"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    data-vize-ui="listbox-item"
    part="item"
    :data-state="itemState"
    :data-value="value"
    :data-selected="selected ? 'true' : 'false'"
    :data-active="active ? 'true' : undefined"
    :data-disabled="itemDisabled ? 'true' : undefined"
    :data-selection-mode="itemSelectionMode"
  >
    <slot v-bind="slotState" />
    <slot name="indicator" v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
