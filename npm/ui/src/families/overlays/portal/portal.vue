<script setup lang="ts">
import { computed, inject, onMounted, onUnmounted, provide, ref, useTemplateRef } from "vue";

import { portalDepthKey, registerPortalLayer } from "./portal-stack.ts";

const {
  to = "body",
  disabled = false,
  defer = true,
} = defineProps<{
  /**
   * CSS selector or element the content is moved into.
   *
   * @default "body"
   */
  readonly to?: string | HTMLElement;

  /**
   * Render in place instead of moving the content.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Keep content in place until the target exists, avoiding SSR mismatch.
   *
   * @default true
   */
  readonly defer?: boolean;
}>();

defineSlots<{
  /** Portalled contents. */
  default(): unknown;
}>();

// Depth flows through the component tree, so it survives Teleport relocation
// and renders deterministically on the server.
const depth = inject(portalDepthKey, 0);
provide(portalDepthKey, depth + 1);

const hydrated = ref(false);
const element = useTemplateRef<HTMLDivElement>("element");
let releaseLayer: (() => void) | null = null;
onMounted(() => {
  hydrated.value = true;
  if (element.value) releaseLayer = registerPortalLayer({ depth, element: element.value });
});
onUnmounted(() => {
  releaseLayer?.();
  releaseLayer = null;
});

const teleportDisabled = computed(() => disabled || !hydrated.value);

defineExpose({ depth, element });
</script>

<template>
  <div data-vize-ui="portal-host">
    <Teleport :to :disabled="teleportDisabled" :defer>
      <div ref="element" data-vize-ui="portal" :data-vize-portal-depth="depth">
        <slot />
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
