<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { tabsContext } from "./tabs-context.ts";
import type {
  TabsContentExpose,
  TabsContentSlotState,
  TabsContentState,
  TabsOrientation,
} from "./tabs-types.ts";

const {
  value,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /** Content value paired with a TabsTrigger. @default required */
  readonly value: string;

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
  default(props: TabsContentSlotState): unknown;
}>();

const context = tabsContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const contentId = computed(() => context.getContentId(value));
const selected = computed(() => context.isSelected(value));
const contentState = computed<TabsContentState>(() => (selected.value ? "active" : "inactive"));
const labelledBy = computed(() => {
  if (ariaLabel !== undefined) return undefined;
  return ariaLabelledby ?? context.getTriggerId(value);
});
const slotState = computed<TabsContentSlotState>(() => ({
  disabled: context.disabled.value,
  orientation: context.orientation.value,
  selected: selected.value,
  state: contentState.value,
  value,
}));

function focusContent(options?: FocusOptions): void {
  if (selected.value) element.value?.focus(options);
}

type TabsContentSetupExpose = Omit<
  TabsContentExpose,
  keyof TabsContentSlotState | "element" | "id"
> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly orientation: ComputedRef<TabsOrientation>;
  readonly selected: ComputedRef<boolean>;
  readonly state: ComputedRef<TabsContentState>;
  readonly value: string;
};

const exposed = {
  disabled: context.disabled,
  element,
  focusContent,
  id: contentId,
  orientation: context.orientation,
  selected,
  state: contentState,
  value,
} satisfies TabsContentSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="contentId"
    ref="element"
    role="tabpanel"
    :hidden="selected ? undefined : true"
    :tabindex="selected ? 0 : undefined"
    :aria-label="ariaLabel"
    :aria-labelledby="labelledBy"
    :aria-describedby="ariaDescribedby"
    data-vize-ui="tabs-content"
    part="content"
    :data-state="contentState"
    :data-selected="selected ? 'true' : 'false'"
    :data-disabled="context.disabled.value ? 'true' : undefined"
    :data-orientation="context.orientation.value"
    :data-value="value"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
