<script setup lang="ts">
import { computed, onMounted, shallowRef, useTemplateRef, watch, watchEffect } from "vue";
import type { ShallowRef } from "vue";

import { tourContext } from "./tour-context.ts";
import { padTourRect } from "./tour-state.ts";
import type {
  TourSpotlightExpose,
  TourSpotlightRect,
  TourSpotlightSlotState,
} from "./tour-types.ts";

const {
  padding = 4,
  radius = 0,
  interactive = false,
} = defineProps<{
  /**
   * Pixels added around the target box on every side.
   *
   * @default 4
   */
  readonly padding?: number;

  /**
   * Corner radius published as `--vize-ui-tour-spotlight-radius`, in pixels.
   *
   * @default 0
   */
  readonly radius?: number;

  /**
   * Whether the highlighted target stays interactive. Published as `data-interactive` so
   * consumer CSS can cut a pointer hole; the spotlight never blocks input on its own.
   *
   * @default false
   */
  readonly interactive?: boolean;
}>();

defineSlots<{
  /** Optional overlay contents, such as an SVG mask. Receives the padded target box. */
  default(props: TourSpotlightSlotState): unknown;
}>();

const PROPERTIES = ["x", "y", "width", "height"] as const;

const context = tourContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const rect = shallowRef<TourSpotlightRect | null>(null);
const slotState = computed<TourSpotlightSlotState>(() => ({
  rect: rect.value,
  state: context.state.value,
  targetState: context.targetState.value,
}));
let mounted = false;

function publish(host: HTMLDivElement | null, box: TourSpotlightRect | null, corner: number): void {
  if (!host) return;
  for (const key of PROPERTIES) {
    const name = `--vize-ui-tour-target-${key}`;
    if (box === null) host.style.removeProperty(name);
    else host.style.setProperty(name, `${String(box[key])}px`);
  }
  host.style.setProperty("--vize-ui-tour-spotlight-radius", `${String(corner)}px`);
}

function update(): void {
  const target = context.target.value;
  rect.value =
    mounted && context.open.value && target !== null
      ? padTourRect(target.getBoundingClientRect(), padding)
      : null;
}

watch(
  [() => context.open.value, () => context.target.value, () => padding],
  (_values, _previous, onCleanup) => {
    update();
    if (!mounted || rect.value === null || typeof globalThis.addEventListener !== "function") {
      return;
    }
    globalThis.addEventListener("scroll", update, true);
    globalThis.addEventListener("resize", update);
    onCleanup(() => {
      globalThis.removeEventListener("scroll", update, true);
      globalThis.removeEventListener("resize", update);
    });
  },
  { flush: "post" },
);

onMounted(() => {
  mounted = true;
  update();
});

// Target geometry is measurement output published as custom properties imperatively:
// the authoring gate keeps `:style` bindings out of templates.
watchEffect(() => publish(element.value, rect.value, radius), { flush: "post" });

type TourSpotlightSetupExpose = Omit<TourSpotlightExpose, "element" | "rect"> & {
  readonly element: typeof element;
  readonly rect: Readonly<ShallowRef<TourSpotlightRect | null>>;
};

const exposed = {
  element,
  rect,
  update,
} satisfies TourSpotlightSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    aria-hidden="true"
    data-vize-ui="tour-spotlight"
    part="spotlight"
    :hidden="context.open.value ? undefined : true"
    :data-state="context.state.value"
    :data-target="context.targetState.value"
    :data-interactive="interactive ? 'true' : 'false'"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
