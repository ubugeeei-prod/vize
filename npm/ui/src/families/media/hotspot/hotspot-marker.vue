<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import PopoverRoot from "../../overlays/popover/popover-root.vue";
import PopoverTrigger from "../../overlays/popover/popover-trigger.vue";
import { hotspotContext, hotspotMarkerContext } from "./hotspot-context.ts";
import { clampHotspotCoordinate } from "./hotspot-geometry.ts";
import type {
  HotspotMarkerExpose,
  HotspotMarkerSlotState,
  HotspotMarkerState,
} from "./hotspot-types.ts";

const {
  id,
  x,
  y,
  label,
  disabled = false,
} = defineProps<{
  /** Marker id, unique within the root. @default required */
  readonly id: string;

  /** Horizontal position in percent of the image width. @default required */
  readonly x: number;

  /** Vertical position in percent of the image height. @default required */
  readonly y: number;

  /** Accessible name of the marker button. @default required */
  readonly label: string;

  /**
   * Remove the marker from interaction while keeping it rendered.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

defineSlots<{
  /** Visual marker contents rendered inside the button. */
  trigger(props: HotspotMarkerSlotState): unknown;

  /** HotspotContent for this marker. */
  default(props: HotspotMarkerSlotState): unknown;
}>();

const context = hotspotContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const markerId = computed(() => id);
const open = computed(() => !disabled && context.isOpen(id));
const disabledState = computed(() => disabled);
const state = computed<HotspotMarkerState>(() => {
  if (disabled) return "disabled";
  return open.value ? "open" : "closed";
});
const slotState = computed<HotspotMarkerSlotState>(() => ({
  disabled,
  id,
  open: open.value,
  state: state.value,
}));
const markerStyle = computed(() => ({
  "--vize-ui-hotspot-x": `${clampHotspotCoordinate(x)}%`,
  "--vize-ui-hotspot-y": `${clampHotspotCoordinate(y)}%`,
}));

function button(): HTMLButtonElement | null {
  return (
    element.value?.querySelector<HTMLButtonElement>('[data-vize-ui="popover-trigger"]') ?? null
  );
}

function focus(options?: FocusOptions): void {
  button()?.focus(options);
}

function onOpenChange(next: boolean): void {
  context.setOpen(id, next, next ? "marker" : "dismiss");
}

function onTriggerKeydown(event: KeyboardEvent): void {
  if (event.altKey || event.ctrlKey || event.metaKey) return;
  if (context.focusNeighbor(id, event.key)) event.preventDefault();
}

let unregister: (() => void) | null = null;

onMounted(() => {
  unregister = context.registerMarker({
    disabled: () => disabled,
    focus: () => focus(),
    id,
    x: () => clampHotspotCoordinate(x),
    y: () => clampHotspotCoordinate(y),
  });
});

onBeforeUnmount(() => {
  unregister?.();
  unregister = null;
});

hotspotMarkerContext.provide({ disabled: disabledState, id: markerId, open });

type HotspotMarkerSetupExpose = Omit<
  HotspotMarkerExpose,
  keyof HotspotMarkerSlotState | "element"
> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<HotspotMarkerState>;
};

const exposed = {
  disabled: disabledState,
  element,
  focus,
  id: markerId,
  open,
  state,
} satisfies HotspotMarkerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    data-vize-ui="hotspot-marker"
    part="marker"
    :data-state="state"
    :data-id="id"
    :style="markerStyle"
  >
    <PopoverRoot :id="context.getMarkerId(id)" :open :disabled @update:open="onOpenChange">
      <PopoverTrigger :aria-label="label" :disabled @keydown="onTriggerKeydown">
        <slot name="trigger" v-bind="slotState" />
      </PopoverTrigger>
      <slot v-bind="slotState" />
    </PopoverRoot>
  </div>
</template>

<style scoped>
/* Headless by design. Position with var(--vize-ui-hotspot-x) and var(--vize-ui-hotspot-y). */
</style>
