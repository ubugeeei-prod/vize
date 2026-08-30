<script setup lang="ts">
import { computed, nextTick, onUnmounted, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import type { CollectionRegistration } from "../../../collection.ts";
import { toDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { stepperContext, stepperItemContext } from "./stepper-context.ts";
import type {
  StepperItemExpose,
  StepperItemSlotState,
  StepperItemState,
  StepperNavigationMode,
  StepperOrientation,
} from "./stepper-types.ts";

const {
  id = undefined,
  value,
  completed = false,
  disabled = false,
  textValue = undefined,
  order = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Consumer-owned list item id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /** Stable step value used for selection and collection identity. @default required */
  readonly value: string;

  /**
   * Mark this step as completed for styling and linear navigation.
   *
   * @default false
   */
  readonly completed?: boolean;

  /**
   * Disable this step while preserving the rest of the Stepper.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Explicit text used by future collection search or virtualized trigger labels.
   *
   * @default undefined
   */
  readonly textValue?: string | null;

  /**
   * Deterministic order for virtualized, portalled, or server-only steps.
   *
   * @default undefined
   */
  readonly order?: number;

  /**
   * Accessible name when no list item text supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label this list item.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe this list item.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** Step contents. Receives current, completion, and activation state. */
  default(props: StepperItemSlotState): unknown;
}>();

const context = stepperContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const triggerElement = shallowRef<HTMLButtonElement | null>(null);
const itemValue = computed(() => value);
const generatedItemId = computed(() => context.getItemId(value));
const itemId = computed(() =>
  id === null || id === undefined ? generatedItemId.value : toDeterministicId(id),
);
const triggerId = computed(() => context.getTriggerId(value));
const contentId = computed(() => context.getContentId(value));
const completedState = computed(() => completed);
const itemDisabled = computed(() => context.disabled.value || disabled);
const current = computed(() => context.isCurrent(value));
const selectable = computed(() => context.isSelectable(value));
const locked = computed(() => !itemDisabled.value && !selectable.value);
const itemState = computed(() => context.getItemState(value));
const itemIndex = computed(() => context.getItemIndex(value));
const slotState = computed<StepperItemSlotState>(() => ({
  completed: completedState.value,
  current: current.value,
  disabled: itemDisabled.value,
  index: itemIndex.value,
  locked: locked.value,
  navigationMode: context.navigationMode.value,
  orientation: context.orientation.value,
  selectable: selectable.value,
  state: itemState.value,
  value,
}));
let registration: CollectionRegistration<string> | null = null;

function register(): void {
  registration?.unregister();
  registration = context.registerItem({
    completed: completedState,
    contentId,
    disabled: itemDisabled,
    element: triggerElement,
    id: itemId,
    order: () => order,
    textValue: () => textValue,
    triggerId,
    value,
  });
}

watch(() => value, register, { flush: "sync", immediate: true });
watch([itemDisabled, completedState, () => order], () => context.syncActiveValue(), {
  flush: "post",
});
onUnmounted(() => {
  registration?.unregister();
  registration = null;
  void nextTick(context.syncActiveValue);
});

function setTriggerElement(nextElement: HTMLButtonElement | null): void {
  triggerElement.value = nextElement;
}

function focus(options?: FocusOptions): boolean {
  return context.focusValue(value, options);
}

function select(nativeEvent: Event | null = null): boolean {
  return context.selectValue(value, nativeEvent);
}

const itemContext = stepperItemContext.provide({
  completed: completedState,
  contentId,
  current,
  disabled: itemDisabled,
  focus,
  id: itemId,
  index: itemIndex,
  locked,
  navigationMode: context.navigationMode,
  orientation: context.orientation,
  selectable,
  select,
  setTriggerElement,
  state: itemState,
  triggerId,
  value: itemValue,
});

type StepperItemSetupExpose = Omit<
  StepperItemExpose,
  keyof StepperItemSlotState | "contentId" | "element" | "id" | "triggerId"
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
  readonly triggerId: ComputedRef<string>;
  readonly value: ComputedRef<string>;
};

const exposed = {
  completed: completedState,
  contentId,
  current,
  disabled: itemDisabled,
  element,
  focus,
  id: itemId,
  index: itemIndex,
  locked,
  navigationMode: context.navigationMode,
  orientation: context.orientation,
  selectable,
  select,
  state: itemState,
  triggerId,
  value: itemValue,
} satisfies StepperItemSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="itemContext.id.value"
    ref="element"
    role="listitem"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    data-vize-ui="stepper-item"
    part="item"
    :data-state="itemContext.state.value"
    :data-value="itemContext.value.value"
    :data-index="itemContext.index.value"
    :data-current="itemContext.current.value ? 'true' : undefined"
    :data-completed="itemContext.completed.value ? 'true' : undefined"
    :data-disabled="itemContext.disabled.value ? 'true' : undefined"
    :data-selectable="itemContext.selectable.value ? 'true' : 'false'"
    :data-locked="itemContext.locked.value ? 'true' : undefined"
    :data-orientation="itemContext.orientation.value"
    :data-navigation-mode="itemContext.navigationMode.value"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
