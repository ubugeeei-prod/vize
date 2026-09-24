<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { createDismissableLayer } from "../../overlays/dismissable-layer/dismissable-layer.ts";
import type {
  SpeedDialContentExpose,
  SpeedDialDirection,
  SpeedDialSlotState,
  SpeedDialState,
} from "./floating-action-button-types.ts";
import { speedDialContext } from "./speed-dial-context.ts";

const { ariaLabel = undefined } = defineProps<{
  /**
   * Accessible name for the action menu. `undefined` labels it with the trigger.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** SpeedDialAction children. Receives the speed-dial state. */
  default(props: SpeedDialSlotState): unknown;
}>();

const context = speedDialContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const orientation = computed(() =>
  context.direction.value === "up" || context.direction.value === "down"
    ? "vertical"
    : "horizontal",
);
const containerProps = computed(() => context.navigation.getContainerProps());
const dismissableLayer = createDismissableLayer({
  root: element,
  branches: () =>
    [context.triggerElement.value].filter((value): value is HTMLButtonElement => !!value),
  enabled: () => context.open.value,
  escapeKey: true,
  outsideFocus: false,
  outsidePointerDown: true,
  onDismiss: (event) => {
    context.close({ focusTrigger: event.reason === "escape-key" }, event.originalEvent);
  },
});
let mounted = false;

function syncLayer(): void {
  if (mounted && context.open.value && element.value) dismissableLayer.activate();
  else dismissableLayer.deactivate();
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === "Tab") {
    context.close({}, event);
    return;
  }
  containerProps.value.onKeydown(event);
}

const menuRole = "menu" as const;
const menuProps = computed(() => ({
  ...dismissableLayer.layerProps,
  onKeydown,
  role: menuRole,
  tabindex: -1 as const,
}));

watch(
  element,
  (next, previous) => {
    if (previous && context.contentElement.value === previous) context.contentElement.value = null;
    if (next) context.contentElement.value = next;
    syncLayer();
  },
  { flush: "post" },
);
watch(() => context.open.value, syncLayer, { flush: "post" });

onMounted(() => {
  mounted = true;
  syncLayer();
});

onUnmounted(() => {
  mounted = false;
  dismissableLayer.dispose();
  if (context.contentElement.value === element.value) context.contentElement.value = null;
});

type SpeedDialContentSetupExpose = Omit<
  SpeedDialContentExpose,
  "direction" | "disabled" | "element" | "open" | "state"
> & {
  readonly direction: ComputedRef<SpeedDialDirection>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<SpeedDialState>;
};

const exposed = {
  direction: context.direction,
  disabled: context.disabled,
  element,
  open: context.open,
  state: context.state,
} satisfies SpeedDialContentSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="context.contentId.value"
    ref="element"
    v-bind="menuProps"
    :aria-orientation="orientation"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabel ? undefined : context.triggerId.value"
    :hidden="context.open.value ? undefined : true"
    data-vize-ui="speed-dial-content"
    part="content"
    :data-state="context.state.value"
    :data-direction="context.direction.value"
  >
    <slot
      :direction="context.direction.value"
      :disabled="context.disabled.value"
      :open="context.open.value"
      :state="context.state.value"
    />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
