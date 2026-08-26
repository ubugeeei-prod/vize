<script setup lang="ts">
import { computed, onMounted, ref, useTemplateRef } from "vue";

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

const hydrated = ref(false);
onMounted(() => {
  hydrated.value = true;
});

const teleportDisabled = computed(() => disabled || !hydrated.value);
const element = useTemplateRef<HTMLDivElement>("element");

defineExpose({ element });
</script>

<template>
  <div data-vize-ui="portal-host">
    <Teleport :to :disabled="teleportDisabled" :defer>
      <div ref="element" data-vize-ui="portal">
        <slot />
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
