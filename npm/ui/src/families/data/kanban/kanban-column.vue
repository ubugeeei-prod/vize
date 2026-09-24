<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef } from "vue";

import { kanbanContext } from "./kanban-context.ts";

const {
  columnId,
  title,
  count = 0,
  limit = undefined,
  disabled = false,
} = defineProps<{
  /**
   * Column id.
   *
   * @default undefined
   */
  readonly columnId: string;

  /**
   * Column title and accessible name.
   *
   * @default undefined
   */
  readonly title: string;

  /**
   * Number of cards, published as `aria-setsize` context and `data-count`.
   *
   * @default 0
   */
  readonly count?: number;

  /**
   * WIP limit, published as `data-limit`.
   *
   * @default undefined
   */
  readonly limit?: number;

  /**
   * Refuse drops.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

defineSlots<{
  /** Column header contents. */
  header(): unknown;
  /** Cards (KanbanCard elements). */
  default(): unknown;
}>();

const board = kanbanContext.use();
const list = useTemplateRef<HTMLUListElement>("list");
const headingId = computed(() => `${board.id.value}-${columnId}-title`.replace(/\s/gu, "_"));
const target = board.dnd.registerTarget({
  key: `column:${columnId}`,
  element: list,
  edges: ["inside"],
  label: () => `end of ${title}`,
  isDisabled: () => disabled,
  accepts: (payload) => payload !== null && board.accepts(payload.data, columnId),
  onDrop: (event) => board.drop(event.sourceKey, { kind: "column", columnId }),
});
onScopeDispose(() => target.dispose());

const over = computed(() => board.dnd.indicator.value?.targetKey === `column:${columnId}`);
const full = computed(() => limit !== undefined && count >= limit);
</script>

<template>
  <section
    role="group"
    :aria-labelledby="headingId"
    data-vize-ui="kanban-column"
    part="column"
    :data-column-id="columnId"
    :data-count="count"
    :data-limit="limit"
    :data-full="full ? 'true' : undefined"
    :data-disabled="disabled ? 'true' : undefined"
    :data-drop-target="over ? 'true' : undefined"
  >
    <div :id="headingId" data-vize-ui="kanban-column-header" part="column-header">
      <slot name="header" />
    </div>
    <ul ref="list" :aria-labelledby="headingId" data-vize-ui="kanban-list" part="list">
      <slot />
    </ul>
  </section>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
