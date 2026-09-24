<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { menuItemIndicatorContext } from "./menu-context.ts";
import { createMenuSelectEvent, useMenuItem } from "./menu-item-runtime.ts";
import type {
  MenuCheckboxItemExpose,
  MenuCheckboxItemSlotState,
  MenuCheckedState,
  MenuItemCheckedState,
  MenuSelectEvent,
} from "./menu-types.ts";

const {
  modelValue = undefined,
  defaultValue = false,
  disabled = false,
  textValue = undefined,
  closeOnSelect = true,
} = defineProps<{
  /**
   * Controlled checked value (`v-model`). `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: MenuCheckedState;

  /**
   * Initial checked value for uncontrolled use.
   *
   * @default false
   */
  readonly defaultValue?: MenuCheckedState;

  /**
   * Block activation while keeping the item focusable.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Typeahead text when the rendered text is not representative.
   *
   * @default undefined
   */
  readonly textValue?: string;

  /**
   * Close the whole menu tree after an unprevented selection.
   *
   * @default true
   */
  readonly closeOnSelect?: boolean;
}>();

const emit = defineEmits<{
  /** Fired with the toggled checked value (supports `v-model`). */
  "update:modelValue": [value: boolean];
  /** Fired when the item is activated, before toggling; call `preventDefault()` to keep the menu open. */
  select: [event: MenuSelectEvent];
}>();

defineSlots<{
  /** Item contents. Receives checked, highlight, and disabled state. */
  default(props: MenuCheckboxItemSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const checkedState = useControllableState<MenuCheckedState>({
  value: () => modelValue,
  defaultValue: () => defaultValue,
});
const state = computed<MenuItemCheckedState>(() => {
  const value = checkedState.value.value;
  if (value === "indeterminate") return "indeterminate";
  return value ? "checked" : "unchecked";
});
const ariaChecked = computed(() =>
  state.value === "indeterminate" ? "mixed" : state.value === "checked" ? "true" : "false",
);

function activate(event: Event | null): boolean {
  if (item.disabled.value) return false;
  const selectEvent = createMenuSelectEvent(element.value, event);
  emit("select", selectEvent);
  const next = checkedState.value.value !== true;
  checkedState.set(next);
  emit("update:modelValue", next);
  if (!selectEvent.defaultPrevented && closeOnSelect) item.tree.closeAll(event);
  return true;
}

const item = useMenuItem({
  role: "menuitemcheckbox",
  element,
  hint: "menu-checkbox-item",
  disabled: () => disabled,
  textValue: () => textValue,
  activate,
});

menuItemIndicatorContext.provide({ state });

defineExpose({
  checked: checkedState.value,
  disabled: item.disabled,
  element,
  focus: item.focus,
  highlighted: item.highlighted,
  id: item.id,
  select: (event: Event | null = null) => activate(event),
} satisfies Omit<
  MenuCheckboxItemExpose,
  "checked" | "disabled" | "element" | "highlighted" | "id"
> & {
  readonly checked: typeof checkedState.value;
  readonly disabled: typeof item.disabled;
  readonly element: typeof element;
  readonly highlighted: typeof item.highlighted;
  readonly id: typeof item.id;
});
</script>

<template>
  <div
    v-bind="item.interactiveProps"
    :id="item.id.value"
    ref="element"
    :aria-checked="ariaChecked"
    :aria-disabled="item.disabled.value ? 'true' : undefined"
    data-vize-ui="menu-checkbox-item"
    part="item"
    :data-state="state"
    :data-disabled="item.disabled.value ? 'true' : undefined"
    :data-highlighted="item.highlighted.value ? 'true' : undefined"
  >
    <slot
      :checked="checkedState.value.value"
      :disabled="item.disabled.value"
      :highlighted="item.highlighted.value"
      :state="state"
    />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
