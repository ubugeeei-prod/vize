<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { stepperContext } from "./stepper-context.ts";
import type {
  StepperContentExpose,
  StepperContentRole,
  StepperContentSlotState,
  StepperContentState,
  StepperOrientation,
} from "./stepper-types.ts";

const {
  value,
  role = "region",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /** Content value paired with a StepperItem. @default required */
  readonly value: string;

  /**
   * Optional landmark role for the content. `null` renders a plain `div`.
   *
   * @default "region"
   */
  readonly role?: StepperContentRole | null;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the panel. `null` omits the default trigger id.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string | null;

  /**
   * Space-separated ids that describe the panel.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** Panel contents. Receives current visibility, orientation, and availability state. */
  default(props: StepperContentSlotState): unknown;
}>();

const context = stepperContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const contentId = computed(() => context.getContentId(value));
const triggerId = computed(() => context.getTriggerId(value));
const current = computed(() => context.isCurrent(value));
const completed = computed(() => context.isCompleted(value));
const disabled = computed(() => context.isStepDisabled(value));
const contentState = computed<StepperContentState>(() => (current.value ? "active" : "inactive"));
const roleValue = computed(() => role ?? undefined);
const labelledBy = computed(() => {
  if (roleValue.value === undefined || ariaLabel !== undefined) return undefined;
  return ariaLabelledby ?? triggerId.value;
});
const slotState = computed<StepperContentSlotState>(() => ({
  active: current.value,
  completed: completed.value,
  current: current.value,
  disabled: disabled.value,
  orientation: context.orientation.value,
  state: contentState.value,
  value,
}));

function focusContent(options?: FocusOptions): void {
  if (current.value) element.value?.focus(options);
}

type StepperContentSetupExpose = Omit<
  StepperContentExpose,
  keyof StepperContentSlotState | "element" | "id" | "triggerId"
> & {
  readonly active: ComputedRef<boolean>;
  readonly completed: ComputedRef<boolean>;
  readonly current: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly orientation: ComputedRef<StepperOrientation>;
  readonly state: ComputedRef<StepperContentState>;
  readonly triggerId: ComputedRef<string>;
  readonly value: string;
};

const exposed = {
  active: current,
  completed,
  current,
  disabled,
  element,
  focusContent,
  id: contentId,
  orientation: context.orientation,
  state: contentState,
  triggerId,
  value,
} satisfies StepperContentSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="contentId"
    ref="element"
    :role="roleValue"
    :hidden="current ? undefined : true"
    :tabindex="current ? 0 : undefined"
    :aria-label="roleValue === undefined ? undefined : ariaLabel"
    :aria-labelledby="labelledBy"
    :aria-describedby="roleValue === undefined ? undefined : ariaDescribedby"
    data-vize-ui="stepper-content"
    part="content"
    :data-state="contentState"
    :data-value="value"
    :data-current="current ? 'true' : undefined"
    :data-completed="completed ? 'true' : undefined"
    :data-disabled="disabled ? 'true' : undefined"
    :data-orientation="context.orientation.value"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
