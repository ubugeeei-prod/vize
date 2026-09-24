<script setup lang="ts" generic="T">
import { computed, onUnmounted, useTemplateRef, watch } from "vue";

import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { mentionContext } from "./mention-context.ts";
import type { MentionItemSlotState } from "./mention-types.ts";

const {
  id = undefined,
  value,
  disabled = false,
  textValue = undefined,
} = defineProps<{
  /**
   * Consumer-owned option id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /** Item inserted when this option is chosen. @default required */
  readonly value: T;

  /**
   * Disable this option.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Explicit option text. `undefined` extracts the rendered text.
   *
   * @default undefined
   */
  readonly textValue?: string;
}>();

defineSlots<{
  /** Option content. Receives the item, highlight, and disabled state. */
  default?(props: MentionItemSlotState<T>): unknown;
}>();

const context = mentionContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const optionId = useDeterministicId({ id: () => id, hint: "mention-option" });
const itemDisabled = computed(() => context.disabled.value || disabled);
const active = computed(() => context.activeKey.value === optionId.value);
const slotState = computed<MentionItemSlotState<T>>(() => ({
  active: active.value,
  disabled: itemDisabled.value,
  value,
}));
const handlers = computed(() => ({
  role: "option" as const,
  onClick: (event: MouseEvent) => {
    if (!itemDisabled.value) context.choose(value, event);
  },
  onPointermove: (event: PointerEvent) => {
    if (!itemDisabled.value && !active.value && event.pointerType !== "touch") {
      context.highlight(optionId.value);
    }
  },
}));
let registration: CollectionRegistration<string> | null = null;

watch(
  [() => value, optionId],
  () => {
    registration?.unregister();
    registration = context.registerItem({
      disabled: itemDisabled,
      element,
      id: optionId,
      textValue: () => textValue,
      value,
    });
  },
  { flush: "sync", immediate: true },
);

onUnmounted(() => {
  registration?.unregister();
  registration = null;
});

defineExpose({ active, element, select: () => context.choose(value, null) });
</script>

<template>
  <div
    :id="optionId"
    ref="element"
    v-bind="handlers"
    :aria-selected="active ? 'true' : 'false'"
    :aria-disabled="itemDisabled ? 'true' : undefined"
    data-vize-ui="mention-item"
    part="item"
    :data-highlighted="active ? 'true' : undefined"
    :data-disabled="itemDisabled ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
