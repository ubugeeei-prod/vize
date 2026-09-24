<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef } from "vue";

import { kanbanContext } from "./kanban-context.ts";

const {
  cardKey,
  columnId,
  index,
  label = undefined,
  disabled = false,
} = defineProps<{
  /**
   * Stable card key.
   *
   * @default undefined
   */
  readonly cardKey: string;

  /**
   * Column the card is in.
   *
   * @default undefined
   */
  readonly columnId: string;

  /**
   * Zero-based index, published as `aria-posinset` (one-based).
   *
   * @default undefined
   */
  readonly index: number;

  /**
   * Name used by drag announcements; defaults to the rendered text.
   *
   * @default undefined
   */
  readonly label?: string;

  /**
   * Keep the card focusable but not draggable.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

defineSlots<{
  /** Card contents. Receives the drag state. */
  default(props: { readonly dragging: boolean }): unknown;
}>();

const board = kanbanContext.use();
const element = useTemplateRef<HTMLLIElement>("element");
const name = () => label ?? element.value?.textContent?.trim() ?? cardKey;
const source = board.dnd.registerSource({
  key: cardKey,
  element,
  payload: { kind: "vize-kanban-card", data: cardKey, plainText: cardKey },
  label: name,
  isDisabled: () => disabled,
});
const target = board.dnd.registerTarget({
  key: `card:${cardKey}`,
  element,
  edges: ["top", "bottom"],
  label: () => `before ${name()}`,
  accepts: (payload) => payload !== null && board.accepts(payload.data, columnId),
  onDrop: (event) => board.drop(event.sourceKey, { kind: "card", cardKey, edge: event.edge }),
});
onScopeDispose(() => {
  source.dispose();
  target.dispose();
});

const dragging = computed(() => source.isDragging.value);
const dropEdge = computed(() =>
  board.dnd.indicator.value?.targetKey === `card:${cardKey}`
    ? board.dnd.indicator.value.edge
    : null,
);

function onKeydown(event: KeyboardEvent): void {
  source.sourceProps.onKeydown(event);
  if (!event.defaultPrevented) board.navigate(cardKey, event);
}

const cardProps = computed(() => ({
  ...source.sourceProps,
  role: "listitem",
  tabindex: board.tabStop.value === cardKey ? 0 : -1,
  "aria-roledescription": "draggable card",
  "aria-describedby": board.instructionsId.value,
  onKeydown,
  onFocus: () => {
    board.activeCard.value = cardKey;
  },
}));
</script>

<template>
  <li
    v-bind="cardProps"
    ref="element"
    :aria-posinset="index + 1"
    :aria-disabled="disabled ? 'true' : undefined"
    data-vize-ui="kanban-card"
    part="card"
    :data-card-key="cardKey"
    :data-column-id="columnId"
    :data-dragging="dragging ? 'true' : undefined"
    :data-drop-edge="dropEdge ?? undefined"
  >
    <slot :dragging />
  </li>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
