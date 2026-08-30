<script setup lang="ts">
import { computed, nextTick, onUnmounted, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import type { CollectionRegistration } from "./collection.ts";
import { tabsContext } from "./tabs-context.ts";
import type {
  TabsActivationMode,
  TabsOrientation,
  TabsTriggerExpose,
  TabsTriggerSlotState,
  TabsTriggerState,
} from "./tabs-types.ts";

const {
  value,
  type = "button",
  disabled = false,
  textValue = undefined,
  order = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /** Trigger value used by the Tabs selection model. @default required */
  readonly value: string;

  /**
   * Native button submission behavior.
   *
   * @default "button"
   */
  readonly type?: "button" | "reset" | "submit";

  /**
   * Disable this trigger while preserving the current selected panel.
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
   * Deterministic order for virtualized, portalled, or server-only triggers.
   *
   * @default undefined
   */
  readonly order?: number;

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
  /** Trigger contents. Receives current selection, orientation, and availability state. */
  default(props: TabsTriggerSlotState): unknown;

  /** Optional indicator slot for consumer-owned active-marker rendering. */
  indicator(props: TabsTriggerSlotState): unknown;
}>();

const context = tabsContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const triggerId = computed(() => context.getTriggerId(value));
const contentId = computed(() => context.getContentId(value));
const selected = computed(() => context.isSelected(value));
const triggerDisabled = computed(() => context.disabled.value || disabled);
const triggerState = computed<TabsTriggerState>(() =>
  context.getTriggerState(value, triggerDisabled.value),
);
const navigationProps = computed(() => context.navigation.getItemProps(value));
const tabIndex = computed(() =>
  triggerDisabled.value ? undefined : navigationProps.value.tabindex,
);
const slotState = computed<TabsTriggerSlotState>(() => ({
  activationMode: context.activationMode.value,
  disabled: triggerDisabled.value,
  orientation: context.orientation.value,
  selected: selected.value,
  state: triggerState.value,
  value,
}));
let registration: CollectionRegistration<string> | null = null;

function register(): void {
  registration?.unregister();
  registration = context.registerTrigger({
    disabled: triggerDisabled,
    element,
    order: () => order,
    textValue: () => textValue,
    value,
  });
}

watch(() => value, register, { flush: "sync", immediate: true });
watch([triggerDisabled, () => order], () => context.syncActiveValue(), { flush: "post" });
onUnmounted(() => {
  registration?.unregister();
  registration = null;
  void nextTick(context.syncActiveValue);
});

function onPointerdown(event: PointerEvent): void {
  if (!triggerDisabled.value) navigationProps.value.onPointerdown(event);
}

function onFocus(event: FocusEvent): void {
  if (!triggerDisabled.value) navigationProps.value.onFocus(event);
}

function onKeydown(event: KeyboardEvent): void {
  if (!triggerDisabled.value) context.navigation.getContainerProps().onKeydown(event);
}

function onClick(event: MouseEvent): void {
  if (triggerDisabled.value) {
    event.preventDefault();
    event.stopImmediatePropagation();
    return;
  }
  emit("click", event);
  if (!event.defaultPrevented) context.selectValue(value, event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type TabsTriggerSetupExpose = Omit<
  TabsTriggerExpose,
  keyof TabsTriggerSlotState | "element" | "id"
> & {
  readonly activationMode: ComputedRef<TabsActivationMode>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly orientation: ComputedRef<TabsOrientation>;
  readonly selected: ComputedRef<boolean>;
  readonly state: ComputedRef<TabsTriggerState>;
  readonly value: string;
};

const exposed = {
  activationMode: context.activationMode,
  disabled: triggerDisabled,
  element,
  focus,
  id: triggerId,
  orientation: context.orientation,
  selected,
  state: triggerState,
  value,
} satisfies TabsTriggerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    :id="triggerId"
    ref="element"
    :type
    role="tab"
    :disabled="triggerDisabled"
    :tabindex="tabIndex"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-selected="selected ? 'true' : 'false'"
    :aria-controls="contentId"
    data-vize-ui="tabs-trigger"
    part="trigger"
    :data-state="triggerState"
    :data-selected="selected ? 'true' : 'false'"
    :data-disabled="triggerDisabled ? 'true' : undefined"
    :data-orientation="context.orientation.value"
    :data-value="value"
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
