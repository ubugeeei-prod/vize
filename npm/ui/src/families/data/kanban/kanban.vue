<script setup lang="ts" generic="Card, ColumnId extends string">
import { computed, nextTick, shallowRef, useTemplateRef } from "vue";

import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import {
  defaultDragAnnouncements,
  useDragAndDrop,
} from "../../interaction/drag-and-drop/drag-and-drop.ts";
import KanbanCard from "./kanban-card.vue";
import KanbanColumnPart from "./kanban-column.vue";
import { kanbanContext } from "./kanban-context.ts";
import type { KanbanDropTarget } from "./kanban-context.ts";
import type {
  KanbanAnnouncements,
  KanbanBoard,
  KanbanCardSlotProps,
  KanbanColumn,
  KanbanColumnSlotProps,
  KanbanDragData,
  KanbanExpose,
  KanbanLocation,
  KanbanMoveEvent,
} from "./kanban-types.ts";

const {
  columns,
  modelValue,
  getCardKey,
  getCardLabel = undefined,
  canMove = undefined,
  id = undefined,
  dir = "ltr",
  instructions = "Press Space or Enter to pick up the card. Use the arrow keys to choose a position and press Space or Enter to drop it, or Escape to cancel.",
  announcements = undefined,
  ariaLabel = undefined,
} = defineProps<{
  /**
   * Board columns in display order.
   *
   * @default undefined
   */
  readonly columns: readonly KanbanColumn<ColumnId>[];

  /**
   * Cards per column (`v-model`).
   *
   * @default undefined
   */
  readonly modelValue: KanbanBoard<Card, ColumnId>;

  /**
   * Stable key per card.
   *
   * @default undefined
   */
  readonly getCardKey: (card: Card) => string;

  /**
   * Card name for announcements; defaults to the rendered text.
   *
   * @default undefined
   */
  readonly getCardLabel?: (card: Card) => string;

  /**
   * Veto a move before it is offered as a drop target.
   *
   * @default undefined
   */
  readonly canMove?: (card: Card, from: ColumnId, to: ColumnId) => boolean;

  /**
   * Consumer-owned board id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Reading direction for Left/Right column navigation.
   *
   * @default "ltr"
   */
  readonly dir?: "ltr" | "rtl";

  /**
   * Keyboard instructions referenced by every card's `aria-describedby`.
   *
   * @default "Press Space or Enter to pick up the card. …"
   */
  readonly instructions?: string;

  /**
   * Localized grab, move, drop, and cancel announcements.
   *
   * @default undefined
   */
  readonly announcements?: KanbanAnnouncements;

  /**
   * Accessible name of the board.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired with the board after a move (supports `v-model`). */
  "update:modelValue": [value: KanbanBoard<Card, ColumnId>];
  /** Fired after a move commits with origin and destination. */
  move: [event: KanbanMoveEvent<Card, ColumnId>];
}>();

defineSlots<{
  /** Column header contents. Defaults to the column title. */
  column?(props: KanbanColumnSlotProps<Card, ColumnId>): unknown;
  /** Card contents. Defaults to the card label or key. */
  card?(props: KanbanCardSlotProps<Card, ColumnId>): unknown;
  /** Rendered inside columns without cards. */
  empty?(props: { readonly column: KanbanColumn<ColumnId> }): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const boardId = useDeterministicId({ id: () => id, hint: "kanban" });
const instructionsId = computed(() => `${boardId.value}-instructions`);
const activeCard = shallowRef<string | null>(null);

function cardsOf(columnId: ColumnId): readonly Card[] {
  return modelValue[columnId] ?? [];
}

function locate(cardKey: string): (KanbanLocation<ColumnId> & { readonly card: Card }) | null {
  for (const column of columns) {
    const cards = cardsOf(column.id);
    const index = cards.findIndex((card) => getCardKey(card) === cardKey);
    const card = cards[index];
    if (card !== undefined) return { columnId: column.id, index, card };
  }
  return null;
}

function findColumn(columnId: string): KanbanColumn<ColumnId> | undefined {
  return columns.find((column) => column.id === columnId);
}

const tabStop = computed(() => {
  if (activeCard.value !== null && locate(activeCard.value)) return activeCard.value;
  for (const column of columns) {
    const first = cardsOf(column.id)[0];
    if (first !== undefined) return getCardKey(first);
  }
  return null;
});

function accepts(cardKey: string, columnId: string): boolean {
  const from = locate(cardKey);
  const column = findColumn(columnId);
  if (!from || !column || column.disabled) return false;
  if (
    column.id !== from.columnId &&
    column.limit !== undefined &&
    cardsOf(column.id).length >= column.limit
  ) {
    return false;
  }
  return canMove ? canMove(from.card, from.columnId, column.id) : true;
}

function focusCard(cardKey: string | null): void {
  if (cardKey === null) return;
  activeCard.value = cardKey;
  void nextTick(() => {
    for (const candidate of element.value?.querySelectorAll<HTMLElement>("[data-card-key]") ?? []) {
      if (candidate.dataset.cardKey === cardKey) {
        candidate.focus();
        return;
      }
    }
  });
}

function drop(cardKey: string, target: KanbanDropTarget): void {
  const from = locate(cardKey);
  if (!from) return;
  let toColumn: ColumnId;
  let toIndex: number;
  if (target.kind === "column") {
    const column = findColumn(target.columnId);
    if (!column) return;
    toColumn = column.id;
    toIndex = cardsOf(column.id).filter((card) => getCardKey(card) !== cardKey).length;
  } else {
    const over = locate(target.cardKey);
    if (!over || target.cardKey === cardKey) return;
    toColumn = over.columnId;
    const remaining = cardsOf(over.columnId).filter((card) => getCardKey(card) !== cardKey);
    const overIndex = remaining.findIndex((card) => getCardKey(card) === target.cardKey);
    toIndex = overIndex + (target.edge === "bottom" ? 1 : 0);
  }
  const source = cardsOf(from.columnId).filter((card) => getCardKey(card) !== cardKey);
  const destination = from.columnId === toColumn ? source : [...cardsOf(toColumn)];
  destination.splice(toIndex, 0, from.card);
  const board: KanbanBoard<Card, ColumnId> = Object.assign(
    {},
    modelValue,
    { [from.columnId]: source },
    { [toColumn]: destination },
  );
  if (from.columnId === toColumn && from.index === toIndex) return;
  emit("update:modelValue", board);
  emit("move", {
    card: from.card,
    cardKey,
    from: { columnId: from.columnId, index: from.index },
    to: { columnId: toColumn, index: toIndex },
    board,
  });
  focusCard(cardKey);
}

function navigate(cardKey: string, event: KeyboardEvent): void {
  if (event.target !== event.currentTarget || event.isComposing) return;
  const from = locate(cardKey);
  if (!from) return;
  const columnIndex = columns.findIndex((column) => column.id === from.columnId);
  const forward = dir === "rtl" ? "ArrowLeft" : "ArrowRight";
  const backward = dir === "rtl" ? "ArrowRight" : "ArrowLeft";
  const cards = cardsOf(from.columnId);
  let next: Card | undefined;
  if (event.key === "ArrowDown") next = cards[from.index + 1];
  else if (event.key === "ArrowUp") next = cards[from.index - 1];
  else if (event.key === "Home") next = cards[0];
  else if (event.key === "End") next = cards.at(-1);
  else if (event.key === forward || event.key === backward) {
    const step = event.key === forward ? 1 : -1;
    for (let index = columnIndex + step; index >= 0 && index < columns.length; index += step) {
      const column = columns[index];
      const candidates = column ? cardsOf(column.id) : [];
      if (candidates.length > 0) {
        next = candidates[Math.min(from.index, candidates.length - 1)];
        break;
      }
    }
  } else return;
  event.preventDefault();
  if (next !== undefined) focusCard(getCardKey(next));
}

// Announcements resolve lazily so a changed `announcements` prop applies to the next phase.
const dnd = useDragAndDrop<KanbanDragData>({
  announcements: {
    grab: (context) => (announcements?.grab ?? defaultDragAnnouncements.grab)(context),
    move: (context) => (announcements?.move ?? defaultDragAnnouncements.move)(context),
    drop: (context) => (announcements?.drop ?? defaultDragAnnouncements.drop)(context),
    cancel: (context) => (announcements?.cancel ?? defaultDragAnnouncements.cancel)(context),
  },
});

kanbanContext.provide({
  id: boardId,
  dnd,
  tabStop,
  activeCard,
  instructionsId,
  accepts,
  drop,
  navigate,
});

function cardLabelProps(card: Card): { readonly label?: string } {
  return getCardLabel ? { label: getCardLabel(card) } : {};
}

function columnProps(column: KanbanColumn<ColumnId>): { readonly limit?: number } {
  return column.limit === undefined ? {} : { limit: column.limit };
}

function columnSlotProps(column: KanbanColumn<ColumnId>): KanbanColumnSlotProps<Card, ColumnId> {
  const cards = cardsOf(column.id);
  return { column, cards, full: column.limit !== undefined && cards.length >= column.limit };
}

function cardSlotProps(
  card: Card,
  column: KanbanColumn<ColumnId>,
  index: number,
): KanbanCardSlotProps<Card, ColumnId> {
  const cardKey = getCardKey(card);
  return {
    card,
    cardKey,
    column,
    index,
    dragging: dnd.isDragging.value && dnd.sourceKey.value === cardKey,
  };
}

const exposed = {
  element,
  focusCard: (cardKey?: string) => focusCard(cardKey ?? tabStop.value),
  cancelDrag: () => dnd.cancel(),
} satisfies Omit<KanbanExpose, "element"> & { readonly element: typeof element };

defineExpose(exposed);
</script>

<template>
  <div
    :id="boardId"
    ref="element"
    role="group"
    :aria-label="ariaLabel"
    aria-roledescription="board"
    :dir
    data-vize-ui="kanban"
    part="root"
    :data-dragging="dnd.isDragging.value ? 'true' : undefined"
  >
    <p :id="instructionsId" hidden data-vize-ui="kanban-instructions">{{ instructions }}</p>
    <KanbanColumnPart
      v-for="column in columns"
      :key="column.id"
      v-bind="columnProps(column)"
      :column-id="column.id"
      :title="column.title"
      :count="cardsOf(column.id).length"
      :disabled="column.disabled === true"
    >
      <template #header>
        <slot name="column" v-bind="columnSlotProps(column)">{{ column.title }}</slot>
      </template>
      <KanbanCard
        v-for="(card, index) in cardsOf(column.id)"
        :key="getCardKey(card)"
        v-bind="cardLabelProps(card)"
        :card-key="getCardKey(card)"
        :column-id="column.id"
        :index
      >
        <slot name="card" v-bind="cardSlotProps(card, column, index)">{{
          getCardLabel?.(card) ?? getCardKey(card)
        }}</slot>
      </KanbanCard>
      <li
        v-if="cardsOf(column.id).length === 0"
        role="none"
        data-vize-ui="kanban-empty"
        part="empty"
      >
        <slot name="empty" :column />
      </li>
    </KanbanColumnPart>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
