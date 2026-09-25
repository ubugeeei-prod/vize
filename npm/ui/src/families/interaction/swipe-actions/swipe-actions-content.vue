<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef, watch } from "vue";

import { swipeActionsContext } from "./swipe-actions-context.ts";

const { keyboardHint = "Use arrow keys to reveal actions" } = defineProps<{
  /**
   * Hidden description announcing the keyboard alternative to swiping.
   *
   * @default "Use arrow keys to reveal actions"
   */
  readonly keyboardHint?: string;
}>();

defineSlots<{
  /** The visible row contents that slide to reveal actions. */
  default(props: { readonly open: boolean }): unknown;
}>();

const context = swipeActionsContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const hintId = computed(() => `${context.contentId.value}-hint`);

watch(element, (target) => context.registerContent(target), { flush: "sync", immediate: true });
onScopeDispose(() => context.registerContent(null));

// Bound together: the row is a focusable surface whose arrow keys reveal the trays.
const surfaceProps = computed(() => ({
  tabindex: context.disabled.value ? -1 : 0,
  "aria-describedby": hintId.value,
  "aria-keyshortcuts": "ArrowLeft ArrowRight Escape",
  onKeydown: context.onContentKeydown,
  onPointerdown: context.onPointerdown,
}));
</script>

<template>
  <div
    :id="context.contentId.value"
    ref="element"
    v-bind="surfaceProps"
    part="content"
    data-vize-ui="swipe-actions-content"
    :data-open="context.open.value ?? undefined"
  >
    <slot :open="context.open.value !== null" />
    <span :id="hintId" hidden>{{ keyboardHint }}</span>
  </div>
</template>

<style scoped>
/* Headless by design. Apply transform: translateX(var(--vize-swipe-offset)). */
</style>
