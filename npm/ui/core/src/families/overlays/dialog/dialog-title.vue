<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { toDeterministicId } from "../../../deterministic-id.ts";
import { dialogContext } from "./dialog-context.ts";
import type { DialogTitleExpose } from "./dialog-types.ts";
import type { PrimitiveAs, PrimitiveElement } from "../../../primitive.ts";

const { id = undefined, as = "h2" } = defineProps<{
  /**
   * Consumer-owned title id. `null` and `undefined` use the Dialog default.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Native element, custom element, or component to render.
   *
   * @default "h2"
   */
  readonly as?: PrimitiveAs;
}>();

defineSlots<{
  /** Visible Dialog title. */
  default(): unknown;
}>();

const context = dialogContext.use();
const element = useTemplateRef<PrimitiveElement>("element");
const titleId = computed(() => (id == null ? context.titleId.value : toDeterministicId(id)));

type DialogTitleSetupExpose = Omit<DialogTitleExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
} satisfies DialogTitleSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component :is="as" :id="titleId" ref="element" data-vize-ui="dialog-title" part="title">
    <slot />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
