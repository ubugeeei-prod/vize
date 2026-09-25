<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef } from "vue";

import { actionSheetMenuContext } from "./action-sheet-context.ts";
import type { ActionSheetSelectEvent } from "./action-sheet-types.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";

const {
  value,
  disabled = false,
  destructive = false,
  textValue = undefined,
} = defineProps<{
  /**
   * Action identifier reported by `select`.
   *
   * @default required
   */
  readonly value: string;

  /**
   * Disable the action (skipped by arrow keys and typeahead, announced with `aria-disabled`).
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Mark a destructive action for styling (`data-destructive`).
   *
   * @default false
   */
  readonly destructive?: boolean;

  /**
   * Text used for typeahead when the label is not plain text.
   *
   * @default the element's text content
   */
  readonly textValue?: string;
}>();

const emit = defineEmits<{
  /** Fired when the action is chosen; call `preventDefault()` to keep the sheet open. */
  select: [event: ActionSheetSelectEvent];
}>();

defineSlots<{
  /** Action label and icon. */
  default(props: { readonly disabled: boolean }): unknown;
}>();

const context = actionSheetMenuContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const itemId = useDeterministicId({ hint: "action" });
const unregister = context.register({
  id: itemId.value,
  disabled: () => disabled,
  textValue: () => textValue ?? element.value?.textContent ?? "",
  element: () => element.value,
});
onScopeDispose(unregister);
const tabIndex = computed(() => (!disabled && context.activeId.value === itemId.value ? 0 : -1));

function choose(nativeEvent: Event): void {
  if (disabled) return;
  let prevented = false;
  const event: ActionSheetSelectEvent = {
    value,
    nativeEvent,
    preventDefault: () => {
      prevented = true;
    },
    get defaultPrevented() {
      return prevented;
    },
  };
  emit("select", event);
  if (!prevented) context.close(nativeEvent);
}

function onFocus(): void {
  context.setActive(itemId.value);
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key !== "Enter" && event.key !== " ") return;
  event.preventDefault();
  choose(event);
}

// Roving tabindex and handlers are bound together so only the active action is a tab stop.
const itemProps = computed(() => ({
  role: "menuitem",
  tabindex: tabIndex.value,
  onClick: choose,
  onKeydown,
  onFocus,
}));
</script>

<template>
  <div
    :id="itemId"
    ref="element"
    v-bind="itemProps"
    :aria-disabled="disabled ? 'true' : undefined"
    part="item"
    data-vize-ui="action-sheet-item"
    :data-value="value"
    :data-destructive="destructive ? 'true' : undefined"
    :data-disabled="disabled ? 'true' : undefined"
  >
    <slot :disabled="disabled" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
