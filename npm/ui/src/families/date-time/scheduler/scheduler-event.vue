<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef } from "vue";

import { schedulerContext } from "./scheduler-context.ts";
import type { RowPlacement, SchedulerEvent, TimeGridPlacement } from "./scheduler-layout.ts";

const {
  event,
  placement = undefined,
  row = undefined,
  resizable = true,
} = defineProps<{
  /** Event rendered by this item. @default undefined (required) */
  readonly event: SchedulerEvent<unknown>;
  /** Time-grid placement; drives `--vize-scheduler-*` position variables. @default undefined */
  readonly placement?: TimeGridPlacement<unknown> | undefined;
  /** Row placement for all-day lanes and month weeks. @default undefined */
  readonly row?: RowPlacement<unknown> | undefined;
  /** Render a resize handle at the end of time-grid events. @default true */
  readonly resizable?: boolean;
}>();

defineSlots<{
  /** Event content. Receives the event and its placement; defaults to the title. */
  default(props: {
    readonly event: SchedulerEvent<unknown>;
    readonly placement: TimeGridPlacement<unknown> | undefined;
    readonly row: RowPlacement<unknown> | undefined;
    readonly label: string;
  }): unknown;
}>();

const context = schedulerContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const handle = useTemplateRef<HTMLSpanElement>("handle");
const mode = computed(() => (placement ? "time" : "row"));
const move = context.registerEventSource(() => event.id, "move", element);
const resize = context.registerEventSource(() => event.id, "resize", handle);
const label = computed(() => context.eventLabel(event));
const locked = computed(() => context.slotState.value.readOnly || context.slotState.value.disabled);
const showHandle = computed(() => resizable && placement !== undefined && !locked.value);
const style = computed<Record<string, string>>(() => {
  if (placement) {
    return {
      "--vize-scheduler-top": `${placement.top * 100}%`,
      "--vize-scheduler-height": `${placement.height * 100}%`,
      "--vize-scheduler-column": String(placement.column),
      "--vize-scheduler-columns": String(placement.columns),
      "--vize-scheduler-span": String(placement.span),
    };
  }
  if (row) {
    return {
      "--vize-scheduler-start": String(row.startIndex + 1),
      "--vize-scheduler-end": String(row.endIndex + 2),
      "--vize-scheduler-lane": String(row.lane),
    };
  }
  return {};
});

onScopeDispose(() => {
  move.dispose();
  resize.dispose();
});

function onClick(nativeEvent: MouseEvent): void {
  context.onEventActivate(event.id, nativeEvent);
}

function onKeydown(nativeEvent: KeyboardEvent): void {
  context.onEventKeydown(event.id, mode.value, nativeEvent);
}

function onPointerDown(nativeEvent: PointerEvent): void {
  if (nativeEvent.currentTarget instanceof HTMLElement) {
    context.onEventPointerDown(
      nativeEvent.clientY - nativeEvent.currentTarget.getBoundingClientRect().top,
    );
  }
}

function stop(nativeEvent: Event): void {
  nativeEvent.stopPropagation();
}
</script>

<template>
  <div
    ref="element"
    role="button"
    tabindex="0"
    :aria-label="label"
    :aria-disabled="context.slotState.value.disabled ? 'true' : undefined"
    data-vize-ui="scheduler-event"
    part="event"
    :data-event-id="event.id"
    :data-mode="mode"
    :data-all-day="event.allDay ? 'true' : undefined"
    :data-continues-before="(placement ?? row)?.continuesBefore ? 'true' : undefined"
    :data-continues-after="(placement ?? row)?.continuesAfter ? 'true' : undefined"
    :data-dragging="move.isDragging.value || resize.isDragging.value ? 'true' : undefined"
    :style="style"
    v-bind="locked ? {} : move.sourceProps"
    @click="onClick"
    @keydown="onKeydown"
    @pointerdown="onPointerDown"
  >
    <slot :event="event" :placement="placement" :row="row" :label="label">{{ event.title }}</slot>
    <span
      v-if="showHandle"
      ref="handle"
      aria-hidden="true"
      data-vize-ui="scheduler-event-resize"
      part="resize-handle"
      v-bind="resize.sourceProps"
      @pointerdown="stop"
      @mousedown="stop"
      @touchstart="stop"
      @click="stop"
    ></span>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
