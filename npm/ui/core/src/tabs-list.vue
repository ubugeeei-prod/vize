<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { tabsContext } from "./tabs-context.ts";
import type {
  TabsActivationMode,
  TabsDirection,
  TabsListExpose,
  TabsListSlotState,
  TabsOrientation,
  TabsState,
  TabsValue,
} from "./tabs-types.ts";

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
   * Space-separated ids that label the tablist.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the tablist.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** TabsTrigger children. Receives the current root and list state. */
  default(props: TabsListSlotState): unknown;
}>();

const context = tabsContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const slotState = computed<TabsListSlotState>(() => ({
  activationMode: context.activationMode.value,
  dir: context.dir.value,
  disabled: context.disabled.value,
  listId: context.listId.value,
  orientation: context.orientation.value,
  state: context.state.value,
  value: context.value.value,
}));

type TabsListSetupExpose = Omit<TabsListExpose, keyof TabsListSlotState | "element"> & {
  readonly activationMode: ComputedRef<TabsActivationMode>;
  readonly dir: ComputedRef<TabsDirection>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly listId: ComputedRef<string>;
  readonly orientation: ComputedRef<TabsOrientation>;
  readonly state: ComputedRef<TabsState>;
  readonly value: ComputedRef<TabsValue>;
};

const exposed = {
  activationMode: context.activationMode,
  dir: context.dir,
  disabled: context.disabled,
  element,
  focus: context.focus,
  listId: context.listId,
  orientation: context.orientation,
  state: context.state,
  value: context.value,
} satisfies TabsListSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="context.listId.value"
    ref="element"
    role="tablist"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-orientation="context.orientation.value"
    data-vize-ui="tabs-list"
    part="list"
    :data-state="context.state.value"
    :data-disabled="context.disabled.value ? 'true' : undefined"
    :data-orientation="context.orientation.value"
    :data-activation-mode="context.activationMode.value"
    :data-value="context.value.value ?? undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
