<script setup lang="ts">
import { computed, onUnmounted, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { stepperContext, stepperItemContext } from "./stepper-context.ts";
import type {
  StepperItemState,
  StepperNavigationMode,
  StepperOrientation,
  StepperTriggerExpose,
  StepperTriggerSlotState,
} from "./stepper-types.ts";

const {
  type = "button",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Native button submission behavior.
   *
   * @default "button"
   */
  readonly type?: "button" | "reset" | "submit";

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the trigger.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the trigger.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

const emit = defineEmits<{
  /** Fired before this trigger requests selection. Call `preventDefault()` to keep state unchanged. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Trigger contents. Receives current, completion, and activation state. */
  default(props: StepperTriggerSlotState): unknown;

  /** Optional completion/current indicator controlled by the consumer. */
  indicator(props: StepperTriggerSlotState): unknown;
}>();

const context = stepperContext.use();
const item = stepperItemContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const nativeDisabled = computed(() => item.disabled.value);
const ariaDisabled = computed(() =>
  nativeDisabled.value || !item.selectable.value ? "true" : undefined,
);
const navigationProps = computed(() => context.navigation.getItemProps(item.value.value));
const tabIndex = computed(() =>
  nativeDisabled.value ? undefined : navigationProps.value.tabindex,
);
const slotState = computed<StepperTriggerSlotState>(() => ({
  completed: item.completed.value,
  current: item.current.value,
  disabled: item.disabled.value,
  index: item.index.value,
  locked: item.locked.value,
  navigationMode: item.navigationMode.value,
  orientation: item.orientation.value,
  selectable: item.selectable.value,
  state: item.state.value,
  value: item.value.value,
}));

watch(element, (button) => item.setTriggerElement(button), { flush: "post", immediate: true });
onUnmounted(() => item.setTriggerElement(null));

function onPointerdown(event: PointerEvent): void {
  if (!nativeDisabled.value) navigationProps.value.onPointerdown(event);
}

function onFocus(event: FocusEvent): void {
  if (!nativeDisabled.value) navigationProps.value.onFocus(event);
}

function onKeydown(event: KeyboardEvent): void {
  if (!nativeDisabled.value) context.navigation.getContainerProps().onKeydown(event);
}

function onClick(event: MouseEvent): void {
  if (nativeDisabled.value || !item.selectable.value) {
    event.preventDefault();
    if (nativeDisabled.value) event.stopImmediatePropagation();
    return;
  }
  emit("click", event);
  if (!event.defaultPrevented) item.select(event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

function select(): boolean {
  return item.select(null);
}

type StepperTriggerSetupExpose = Omit<
  StepperTriggerExpose,
  keyof StepperTriggerSlotState | "contentId" | "element" | "id"
> & {
  readonly completed: ComputedRef<boolean>;
  readonly contentId: ComputedRef<string>;
  readonly current: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly index: ComputedRef<number>;
  readonly locked: ComputedRef<boolean>;
  readonly navigationMode: ComputedRef<StepperNavigationMode>;
  readonly orientation: ComputedRef<StepperOrientation>;
  readonly selectable: ComputedRef<boolean>;
  readonly state: ComputedRef<StepperItemState>;
  readonly value: ComputedRef<string>;
};

const exposed = {
  completed: item.completed,
  contentId: item.contentId,
  current: item.current,
  disabled: item.disabled,
  element,
  focus,
  id: item.triggerId,
  index: item.index,
  locked: item.locked,
  navigationMode: item.navigationMode,
  orientation: item.orientation,
  selectable: item.selectable,
  select,
  state: item.state,
  value: item.value,
} satisfies StepperTriggerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    :id="item.triggerId.value"
    ref="element"
    :type
    :disabled="nativeDisabled"
    :tabindex="tabIndex"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-current="item.current.value ? 'step' : undefined"
    :aria-controls="item.contentId.value"
    :aria-disabled="ariaDisabled"
    data-vize-ui="stepper-trigger"
    part="trigger"
    :data-state="item.state.value"
    :data-value="item.value.value"
    :data-index="item.index.value"
    :data-current="item.current.value ? 'true' : undefined"
    :data-completed="item.completed.value ? 'true' : undefined"
    :data-disabled="item.disabled.value ? 'true' : undefined"
    :data-selectable="item.selectable.value ? 'true' : 'false'"
    :data-locked="item.locked.value ? 'true' : undefined"
    :data-orientation="item.orientation.value"
    :data-navigation-mode="item.navigationMode.value"
    @pointerdown="onPointerdown"
    @focus="onFocus"
    @keydown="onKeydown"
    @click="onClick"
  >
    <slot v-bind="slotState" />
    <slot name="indicator" v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
