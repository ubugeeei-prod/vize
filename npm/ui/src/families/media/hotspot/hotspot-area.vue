<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { hotspotContext } from "./hotspot-context.ts";
import { hotspotPolygonPoints, sanitizeHotspotHref } from "./hotspot-geometry.ts";
import type { HotspotShape } from "./hotspot-geometry.ts";
import type { HotspotAreaExpose, HotspotAreaSlotState } from "./hotspot-types.ts";

const {
  id,
  shape,
  label,
  href = undefined,
  disabled = false,
} = defineProps<{
  /** Area id; activating the area makes it the root's active hotspot. @default required */
  readonly id: string;

  /** Region in image-space percent. @default required */
  readonly shape: HotspotShape;

  /** Accessible name of the region. @default required */
  readonly label: string;

  /**
   * Render the region as a link to this URL instead of a toggle button.
   *
   * @default undefined
   */
  readonly href?: string;

  /**
   * Remove the region from interaction.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when a button region is activated, before the active hotspot toggles. */
  activate: [nativeEvent: Event];
}>();

defineSlots<{
  /** Extra SVG content drawn inside the region, e.g. a `<title>`. */
  default(props: HotspotAreaSlotState): unknown;
}>();

const context = hotspotContext.use();
const element = useTemplateRef<SVGSVGElement>("element");
const active = computed(() => context.isOpen(id));
const slotState = computed<HotspotAreaSlotState>(() => ({ active: active.value, id }));
const rect = computed(() => (shape.type === "rect" ? shape : null));
const circle = computed(() => (shape.type === "circle" ? shape : null));
const polygonPoints = computed(() =>
  shape.type === "polygon" ? hotspotPolygonPoints(shape.points) : null,
);
const shapeState = computed(() => shape);
const linkProps = computed<{ readonly href?: string }>(() => {
  const safe = href === undefined ? undefined : sanitizeHotspotHref(href);
  return safe === undefined ? {} : { href: safe };
});

function activate(event: Event): void {
  if (disabled) return;
  emit("activate", event);
  if (event.defaultPrevented) return;
  context.setOpen(id, !context.isOpen(id), "area");
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key !== "Enter" && event.key !== " ") return;
  // Activate first so `defaultPrevented` reflects only the consumer's `activate` listener.
  activate(event);
  event.preventDefault();
}

// Role, tab stop, and activation handlers are bound together as one button contract.
const buttonProps = computed<{
  readonly role: "button";
  readonly tabindex: number;
  readonly onClick: (event: MouseEvent) => void;
  readonly onKeydown: (event: KeyboardEvent) => void;
}>(() => ({
  role: "button",
  tabindex: disabled ? -1 : 0,
  onClick: activate,
  onKeydown,
}));

let unregister: (() => void) | null = null;

onMounted(() => {
  unregister = context.registerArea(id, () => shape);
});

onBeforeUnmount(() => {
  unregister?.();
  unregister = null;
});

type HotspotAreaSetupExpose = Omit<HotspotAreaExpose, "active" | "element" | "id" | "shape"> & {
  readonly active: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly id: string;
  readonly shape: ComputedRef<HotspotShape>;
};

const exposed = { active, element, id, shape: shapeState } satisfies HotspotAreaSetupExpose;

defineExpose(exposed);
</script>

<template>
  <svg
    ref="element"
    viewBox="0 0 100 100"
    preserveAspectRatio="none"
    data-vize-ui="hotspot-area"
    part="area"
    :data-id="id"
    :data-shape="shape.type"
    :data-state="active ? 'active' : 'inactive'"
    :data-disabled="disabled ? 'true' : undefined"
  >
    <component
      is="a"
      v-if="href !== undefined"
      v-bind="linkProps"
      :aria-label="label"
      data-vize-ui="hotspot-area-link"
    >
      <rect v-if="rect" :x="rect.x" :y="rect.y" :width="rect.width" :height="rect.height" />
      <circle v-else-if="circle" :cx="circle.cx" :cy="circle.cy" :r="circle.r" />
      <polygon v-else-if="polygonPoints !== null" :points="polygonPoints" />
      <slot v-bind="slotState" />
    </component>
    <g
      v-else
      v-bind="buttonProps"
      :aria-label="label"
      :aria-pressed="active ? 'true' : 'false'"
      :aria-disabled="disabled ? 'true' : undefined"
      data-vize-ui="hotspot-area-button"
    >
      <rect v-if="rect" :x="rect.x" :y="rect.y" :width="rect.width" :height="rect.height" />
      <circle v-else-if="circle" :cx="circle.cx" :cy="circle.cy" :r="circle.r" />
      <polygon v-else-if="polygonPoints !== null" :points="polygonPoints" />
      <slot v-bind="slotState" />
    </g>
  </svg>
</template>

<style scoped>
/* Headless by design. Stretch the overlay over the image, e.g. position: absolute; inset: 0. */
</style>
