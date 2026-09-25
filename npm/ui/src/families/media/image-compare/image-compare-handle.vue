<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, useTemplateRef } from "vue";

import { imageCompareContext } from "./image-compare-context.ts";
import type { ImageCompareHandleExpose, ImageCompareSlotState } from "./image-compare-types.ts";
import { imageComparePositionForKey } from "./image-compare-value.ts";

const {
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Accessible name. Defaults to `messages.handleLabel`, then "Comparison position".
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the handle; takes precedence over `ariaLabel`.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the handle.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** Handle contents such as a grip icon. Receives the divider state. */
  default(props: ImageCompareSlotState): unknown;
}>();

const context = imageCompareContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const label = computed(() =>
  ariaLabelledby === undefined
    ? (ariaLabel ?? context.messages.value.handleLabel ?? "Comparison position")
    : undefined,
);
const valueText = computed(
  () =>
    context.messages.value.valueText?.(context.position.value) ??
    `${Math.round(context.position.value)}%`,
);

function onKeydown(event: KeyboardEvent): void {
  if (context.disabled.value) return;
  const next = imageComparePositionForKey(event.key, context.position.value, {
    dir: context.dir.value,
    orientation: context.orientation.value,
    pageStep: context.pageStep.value,
    step: context.step.value > 0 ? context.step.value : 1,
  });
  if (next === null) return;
  event.preventDefault();
  context.setPosition(next, "keyboard", event);
}

// Role, tab stop, and keyboard handling are bound together as one slider contract.
const handleProps = computed<{
  readonly role: "slider";
  readonly tabindex: number;
  readonly onKeydown: (event: KeyboardEvent) => void;
}>(() => ({
  role: "slider",
  tabindex: context.disabled.value ? -1 : 0,
  onKeydown,
}));

onMounted(() => {
  context.handleElement.value = element.value;
});

onBeforeUnmount(() => {
  if (context.handleElement.value === element.value) context.handleElement.value = null;
});

const exposed = { element } satisfies {
  readonly element: typeof element;
} & Omit<ImageCompareHandleExpose, "element">;

defineExpose(exposed);
</script>

<template>
  <div
    :id="context.handleId.value"
    ref="element"
    v-bind="handleProps"
    :aria-label="label"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-orientation="context.orientation.value"
    aria-valuemin="0"
    aria-valuemax="100"
    :aria-valuenow="context.position.value"
    :aria-valuetext="valueText"
    :aria-disabled="context.disabled.value ? 'true' : undefined"
    data-vize-ui="image-compare-handle"
    part="handle"
    :data-state="context.state.value"
    :data-orientation="context.orientation.value"
  >
    <slot v-bind="context.slotState.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Position the handle with var(--vize-ui-image-compare-position). */
</style>
