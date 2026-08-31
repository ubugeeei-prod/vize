<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { toDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { dialogContext } from "./dialog-context.ts";
import type { DialogDescriptionExpose } from "./dialog-types.ts";
import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";

const { id = undefined, as = "p" } = defineProps<{
  /**
   * Consumer-owned description id. `null` and `undefined` use the Dialog default.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Native element, custom element, or component to render.
   *
   * @default "p"
   */
  readonly as?: PrimitiveAs;
}>();

defineSlots<{
  /** Visible Dialog description. */
  default(): unknown;
}>();

const context = dialogContext.use();
const element = useTemplateRef<PrimitiveElement>("element");
const descriptionId = computed(() =>
  id == null ? context.descriptionId.value : toDeterministicId(id),
);

type DialogDescriptionSetupExpose = Omit<DialogDescriptionExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
} satisfies DialogDescriptionSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    :id="descriptionId"
    ref="element"
    data-vize-ui="dialog-description"
    part="description"
  >
    <slot />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
