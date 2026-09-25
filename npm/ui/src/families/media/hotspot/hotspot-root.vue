<script setup lang="ts" generic="Data = undefined">
import { computed, shallowRef, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { hotspotContext } from "./hotspot-context.ts";
import type { HotspotContextValue, HotspotRegistration } from "./hotspot-context.ts";
import { hotspotShapeContains, nextHotspotInDirection } from "./hotspot-geometry.ts";
import type { HotspotDirection, HotspotShape } from "./hotspot-geometry.ts";
import type {
  HotspotActiveChangeSource,
  HotspotDefinition,
  HotspotRootExpose,
  HotspotSlotState,
} from "./hotspot-types.ts";

const {
  id = undefined,
  hotspots = undefined,
  active = undefined,
  defaultActive = null,
  exclusive = true,
} = defineProps<{
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Declared markers. Their `data` type flows into the default slot so markers
   * can be rendered with `v-for` while keeping consumer payloads typed.
   *
   * @default []
   */
  readonly hotspots?: readonly HotspotDefinition<Data>[];

  /**
   * Controlled active marker id (`v-model:active`). `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly active?: string | null;

  /**
   * Initially active marker for uncontrolled use.
   *
   * @default null
   */
  readonly defaultActive?: string | null;

  /**
   * Keep at most one marker open. When `false`, markers open independently and
   * `active` tracks the most recently opened one.
   *
   * @default true
   */
  readonly exclusive?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the active marker requests a new controlled value. */
  "update:active": [id: string | null];

  /** Fired after every distinct active-marker change with its source. */
  activeChange: [id: string | null, previous: string | null, source: HotspotActiveChangeSource];
}>();

defineSlots<{
  /** Image, markers, areas, and content. Receives the typed hotspot list and open state. */
  default?(props: HotspotSlotState<Data>): unknown;
}>();

const EMPTY: readonly HotspotDefinition<Data>[] = Object.freeze([]);
const element = useTemplateRef<HTMLDivElement>("element");
const baseId = useDeterministicId({ id: () => id, hint: "hotspot" });
const activeState = useControllableState<string | null>({
  value: () => active,
  defaultValue: () => defaultActive,
});
const activeId = computed(() => activeState.value.value);
const extraOpen = shallowRef<readonly string[]>([]);
const openIds = computed<readonly string[]>(() => {
  if (exclusive) return activeId.value === null ? [] : [activeId.value];
  const ids = extraOpen.value.filter((value) => value !== activeId.value);
  return activeId.value === null ? ids : [...ids, activeId.value];
});
const hotspotList = computed(() => hotspots ?? EMPTY);
const slotState = computed<HotspotSlotState<Data>>(() => ({
  active: activeId.value,
  hotspots: hotspotList.value,
  openIds: openIds.value,
}));
const markers = new Map<string, HotspotRegistration>();
const areas = new Map<string, () => HotspotShape>();

function currentActive(): string | null {
  return activeId.value;
}

function setActive(next: string | null, source: HotspotActiveChangeSource): boolean {
  const previous = currentActive();
  if (previous === next) return false;
  activeState.set(next);
  emit("update:active", next);
  emit("activeChange", next, previous, source);
  return true;
}

function setOpen(markerId: string, open: boolean, source: HotspotActiveChangeSource): boolean {
  const previous = currentActive();
  if (open) {
    if (!exclusive && previous !== null && previous !== markerId) {
      extraOpen.value = [...extraOpen.value.filter((value) => value !== previous), previous];
    }
    return setActive(markerId, source);
  }
  if (previous === markerId) {
    const remaining = exclusive ? [] : extraOpen.value.filter((value) => value !== markerId);
    extraOpen.value = remaining.slice(0, -1);
    return setActive(remaining.at(-1) ?? null, source);
  }
  if (!extraOpen.value.includes(markerId)) return false;
  extraOpen.value = extraOpen.value.filter((value) => value !== markerId);
  return true;
}

function directionOf(key: string): HotspotDirection | null {
  switch (key) {
    case "ArrowRight":
      return "right";
    case "ArrowLeft":
      return "left";
    case "ArrowDown":
      return "down";
    case "ArrowUp":
      return "up";
    default:
      return null;
  }
}

function focusNeighbor(fromId: string, key: string): boolean {
  const direction = directionOf(key);
  if (direction === null) return false;
  const items = [...markers.values()]
    .filter((marker) => marker.id === fromId || !marker.disabled())
    .map((marker) => ({ id: marker.id, x: marker.x(), y: marker.y() }));
  const next = nextHotspotInDirection(items, fromId, direction);
  if (next === null) return false;
  markers.get(next)?.focus();
  return true;
}

function close(markerId?: string): boolean {
  if (markerId !== undefined) return setOpen(markerId, false, "api");
  const hadExtra = extraOpen.value.length > 0;
  extraOpen.value = [];
  return setActive(null, "api") || hadExtra;
}

function hitTest(x: number, y: number): readonly string[] {
  const hits: string[] = [];
  for (const [areaId, shape] of areas) {
    if (hotspotShapeContains(shape(), { x, y })) hits.push(areaId);
  }
  return hits;
}

hotspotContext.provide({
  active: activeId,
  focusNeighbor,
  getMarkerId: (markerId) => deriveDeterministicId(baseId.value, `marker-${markerId}`),
  id: baseId,
  isOpen: (markerId) => openIds.value.includes(markerId),
  registerArea(areaId, shape) {
    areas.set(areaId, shape);
    return () => {
      if (areas.get(areaId) === shape) areas.delete(areaId);
    };
  },
  registerMarker(registration) {
    markers.set(registration.id, registration);
    return () => {
      if (markers.get(registration.id) === registration) markers.delete(registration.id);
    };
  },
  setOpen,
} satisfies HotspotContextValue);

type HotspotRootSetupExpose = Omit<HotspotRootExpose, "active" | "element" | "openIds"> & {
  readonly active: ComputedRef<string | null>;
  readonly element: typeof element;
  readonly openIds: ComputedRef<readonly string[]>;
};

const exposed = {
  active: activeId,
  close,
  element,
  hitTest,
  open: (markerId: string) => setOpen(markerId, true, "api"),
  openIds,
} satisfies HotspotRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    ref="element"
    data-vize-ui="hotspot-root"
    part="root"
    :data-state="openIds.length > 0 ? 'open' : 'closed'"
    :data-active="activeId ?? undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Give the root `position: relative` so markers can use
   left: var(--vize-ui-hotspot-x); top: var(--vize-ui-hotspot-y). */
</style>
