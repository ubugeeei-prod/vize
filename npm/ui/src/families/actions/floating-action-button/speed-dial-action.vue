<script setup lang="ts">
import { computed, onUnmounted, useTemplateRef, watch } from "vue";

import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import type {
  SpeedDialActionExpose,
  SpeedDialActionSlotState,
} from "./floating-action-button-types.ts";
import { speedDialContext } from "./speed-dial-context.ts";

const {
  value,
  label,
  disabled = false,
} = defineProps<{
  /**
   * Action identity reported by `select`.
   *
   * @default required
   */
  readonly value: string;

  /**
   * Accessible name. Speed-dial actions are usually icon-only, so the label is
   * required and also offered to the slot for a visible tooltip.
   *
   * @default required
   */
  readonly label: string;

  /**
   * Skip this action during navigation and ignore activation.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

const emit = defineEmits<{
  /** Fired before the speed dial handles the selection. Call `preventDefault()` to skip it. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Action icon and optional visible label. */
  default(props: SpeedDialActionSlotState): unknown;
}>();

const context = speedDialContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const disabledState = computed(() => disabled || context.disabled.value);
const itemProps = computed(() => context.navigation.getItemProps(value));
const itemId = computed<string | undefined>(() => itemProps.value.id);
const itemTabindex = computed<-1 | 0>(() => itemProps.value.tabindex ?? -1);
const active = computed(() => context.navigation.activeKey.value === value);
const slotState = computed<SpeedDialActionSlotState>(() => ({
  active: active.value,
  direction: context.direction.value,
  disabled: disabledState.value,
  label,
  open: context.open.value,
  state: context.state.value,
  value,
}));
let registration: CollectionRegistration<string> | null = null;

watch(
  () => value,
  (next) => {
    registration?.unregister();
    registration = context.registry.register({
      disabled: disabledState,
      element,
      key: next,
      textValue: () => label,
      value: next,
    });
  },
  { flush: "sync", immediate: true },
);

onUnmounted(() => {
  registration?.unregister();
  registration = null;
});

function onClick(event: MouseEvent): void {
  if (disabledState.value) return;
  emit("click", event);
  if (!event.defaultPrevented) context.select(value, event);
}

function onFocus(event: FocusEvent): void {
  itemProps.value.onFocus(event);
}

function onPointerdown(event: PointerEvent): void {
  itemProps.value.onPointerdown(event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type SpeedDialActionSetupExpose = Omit<SpeedDialActionExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element, focus } satisfies SpeedDialActionSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    :id="itemId"
    ref="element"
    type="button"
    role="menuitem"
    :tabindex="itemTabindex"
    :disabled="disabledState"
    :aria-label="label"
    data-vize-ui="speed-dial-action"
    part="action"
    :data-value="value"
    :data-active="active ? 'true' : undefined"
    :data-disabled="disabledState ? 'true' : undefined"
    @click="onClick"
    @focus="onFocus"
    @pointerdown="onPointerdown"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
