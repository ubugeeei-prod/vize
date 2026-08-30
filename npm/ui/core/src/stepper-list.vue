<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { stepperContext } from "./stepper-context.ts";
import type {
  StepperDirection,
  StepperListExpose,
  StepperListSlotState,
  StepperNavigationMode,
  StepperOrientation,
  StepperRootState,
  StepperValue,
} from "./stepper-types.ts";

const {
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the step list.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the step list.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** StepperItem children. Receives the current root and list state. */
  default(props: StepperListSlotState): unknown;
}>();

const context = stepperContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const slotState = computed<StepperListSlotState>(() => ({
  completedValues: context.completedValues.value,
  currentIndex: context.currentIndex.value,
  dir: context.dir.value,
  disabled: context.disabled.value,
  listId: context.listId.value,
  navigationMode: context.navigationMode.value,
  orientation: context.orientation.value,
  state: context.state.value,
  value: context.value.value,
}));

type StepperListSetupExpose = Omit<StepperListExpose, keyof StepperListSlotState | "element"> & {
  readonly completedValues: ComputedRef<readonly string[]>;
  readonly currentIndex: ComputedRef<number>;
  readonly dir: ComputedRef<StepperDirection>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly listId: ComputedRef<string>;
  readonly navigationMode: ComputedRef<StepperNavigationMode>;
  readonly orientation: ComputedRef<StepperOrientation>;
  readonly state: ComputedRef<StepperRootState>;
  readonly value: ComputedRef<StepperValue>;
};

const exposed = {
  completedValues: context.completedValues,
  currentIndex: context.currentIndex,
  dir: context.dir,
  disabled: context.disabled,
  element,
  focus: context.focus,
  listId: context.listId,
  navigationMode: context.navigationMode,
  orientation: context.orientation,
  state: context.state,
  value: context.value,
} satisfies StepperListSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="context.listId.value"
    ref="element"
    role="list"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-orientation="context.orientation.value"
    data-vize-ui="stepper-list"
    part="list"
    :data-state="context.state.value"
    :data-disabled="context.disabled.value ? 'true' : undefined"
    :data-orientation="context.orientation.value"
    :data-navigation-mode="context.navigationMode.value"
    :data-value="context.value.value ?? undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
