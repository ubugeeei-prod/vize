<script setup lang="ts">
import { computed, shallowRef, triggerRef } from "vue";
import type { CSSProperties } from "vue";

import type { PrimitiveAs } from "../../foundations/primitive/primitive.ts";
import { stickyStackContext } from "./sticky-stack-context.ts";
import type { StickyStackRegistration } from "./sticky-stack-context.ts";
import { computeStickyOffsets, sortByDocumentOrder } from "./sticky-stack-layout.ts";
import type { StickyStackSlotState } from "./sticky-stack-types.ts";

const { as = "div", offset = 0 } = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "div"
   */
  readonly as?: PrimitiveAs;

  /**
   * Distance from the top of the scroll port where the first item sticks
   * (e.g. the height of a fixed app bar).
   *
   * @default 0
   */
  readonly offset?: number;
}>();

defineSlots<{
  /**
   * Page content containing `StickyStackItem`s. Receives the total stack height,
   * which includes the items once they have registered (after the first render).
   */
  default?(props: StickyStackSlotState): unknown;
}>();

const registrations = shallowRef<StickyStackRegistration[]>([]);
const layout = computed(() => {
  const ordered = sortByDocumentOrder(
    registrations.value.map((registration) => ({ registration, element: registration.element() })),
  );
  const offsets = computeStickyOffsets(
    ordered.map(({ registration }) => registration.height()),
    offset,
    ordered.map(({ registration }) => registration.enabled()),
  );
  const tops = new Map<symbol, number>();
  ordered.forEach(({ registration }, index) =>
    tops.set(registration.id, offsets.tops[index] ?? offset),
  );
  return { tops, total: offsets.total };
});
const total = computed(() => layout.value.total);

stickyStackContext.provide({
  register: (registration) => {
    registrations.value.push(registration);
    triggerRef(registrations);
    return () => {
      const index = registrations.value.indexOf(registration);
      if (index === -1) return;
      registrations.value.splice(index, 1);
      triggerRef(registrations);
    };
  },
  topOf: (id) => layout.value.tops.get(id) ?? offset,
  total,
});

const rootStyle = computed<CSSProperties>(() => ({
  "--vize-ui-sticky-stack-height": `${total.value}px`,
}));

/** Re-read item heights (for example after fonts load). */
function refresh(): void {
  triggerRef(registrations);
}

defineExpose({ total, refresh });
</script>

<template>
  <component :is="as" data-vize-ui="sticky-stack" :style="rootStyle">
    <slot :total="total" />
  </component>
</template>

<style scoped>
/* Headless by design. The total stack height is exposed as a custom property. */
</style>
