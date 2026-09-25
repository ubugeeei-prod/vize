<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { lightboxContext } from "./lightbox-context.ts";
import type { LightboxButtonExpose, LightboxIndexSlotState } from "./lightbox-types.ts";

const { index, ariaLabel = undefined } = defineProps<{
  /** Zero-based item index this thumbnail shows. @default required */
  readonly index: number;

  /**
   * Accessible name. Defaults to the `thumbnail` message.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before selection. Call `preventDefault()` to keep the current item. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Thumbnail content, e.g. a small image. */
  default?(props: LightboxIndexSlotState): unknown;
}>();

const context = lightboxContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const current = computed<boolean>(() => context.index.value === index);
const disabled = computed<boolean>(() => index < 0 || index >= context.count.value);
const label = computed<string>(
  () => ariaLabel ?? context.messages.value.thumbnail(index + 1, context.count.value),
);
const slotState = computed<LightboxIndexSlotState>(() => ({ current: current.value, index }));
let unregister: (() => void) | null = null;

function register(): void {
  unregister?.();
  unregister = element.value === null ? null : context.registerThumbnail(index, element.value);
}

onMounted(register);
watch(() => index, register, { flush: "post" });
onBeforeUnmount(() => {
  unregister?.();
  unregister = null;
});

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented) context.goTo(index, "thumbnail");
}

function onKeydown(event: KeyboardEvent): void {
  const rtl = context.dir.value === "rtl";
  const last = context.count.value - 1;
  let target: number;
  if (event.key === (rtl ? "ArrowLeft" : "ArrowRight")) {
    target = index >= last ? (context.loop.value ? 0 : last) : index + 1;
  } else if (event.key === (rtl ? "ArrowRight" : "ArrowLeft")) {
    target = index <= 0 ? (context.loop.value ? last : 0) : index - 1;
  } else if (event.key === "Home") {
    target = 0;
  } else if (event.key === "End") {
    target = last;
  } else {
    return;
  }
  event.preventDefault();
  context.goTo(target, "thumbnail");
  void nextTick(() => context.focusThumbnail(target));
}

type LightboxThumbnailSetupExpose = Omit<LightboxButtonExpose, "disabled" | "element"> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
};

const exposed = { disabled, element } satisfies LightboxThumbnailSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    role="tab"
    :disabled
    :tabindex="current ? 0 : -1"
    :aria-label="label"
    :aria-selected="current ? 'true' : 'false'"
    :aria-controls="context.getItemId(context.index.value)"
    data-vize-ui="lightbox-thumbnail"
    part="thumbnail"
    :data-state="current ? 'active' : 'inactive'"
    :data-index="index"
    @click="onClick"
    @keydown="onKeydown"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
