<script setup lang="ts">
import { computed, onUnmounted, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import type { PrimitiveElement } from "../../foundations/primitive/primitive.ts";
import type { ToolbarItemExpose } from "./toolbar-contracts.ts";
import { toolbarContext } from "./toolbar-context.ts";
import type { ToolbarNavigationIntent } from "./toolbar-context.ts";
import type {
  ToolbarItemEmits,
  ToolbarItemProps,
  ToolbarItemSlotState,
  ToolbarItemState,
} from "./toolbar-types.ts";

type ToolbarKeyboardPhase = "keydown" | "keyup";
type ToolbarKeyboardAction = "activate" | "prevent" | "ignore";

function getToolbarKeyboardAction(key: string, phase: ToolbarKeyboardPhase): ToolbarKeyboardAction {
  if (key === "Enter") return phase === "keydown" ? "activate" : "ignore";
  if (key === " ") return phase === "keydown" ? "prevent" : "activate";
  return "ignore";
}

const {
  as = "button",
  native = undefined,
  type = "button",
  value,
  disabled = false,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<ToolbarItemProps>();

defineSlots<{
  /** Renders item contents with current availability and navigation state. */
  default(props: ToolbarItemSlotState): unknown;
}>();

const emit = defineEmits<ToolbarItemEmits>();

const context = toolbarContext.use();
const element = useTemplateRef<PrimitiveElement>("element");
const isNativeButton = computed(() => native ?? as === "button");
const itemDisabled = computed(() => context.disabled.value || disabled);
const itemState = computed<ToolbarItemState>(() => context.getItemState(itemDisabled.value));
const tabIndex = computed(() => {
  if (itemDisabled.value) return isNativeButton.value ? undefined : -1;
  if (!context.rovingFocus.value) return isNativeButton.value ? undefined : 0;
  return context.activeValue.value === value ? 0 : -1;
});
const slotState = computed<ToolbarItemSlotState>(() => ({
  dir: context.dir.value,
  disabled: itemDisabled.value,
  orientation: context.orientation.value,
  state: itemState.value,
  value,
}));

const registration = {
  disabled: itemDisabled,
  element,
  value: () => value,
};
const unregister = context.registerItem(registration);

watch([itemDisabled, () => value], () => context.syncActiveValue(registration), { flush: "sync" });
onUnmounted(unregister);

function onClick(event: MouseEvent): void {
  if (itemDisabled.value) {
    event.preventDefault();
    event.stopImmediatePropagation();
    return;
  }
  context.requestItemPress(value, event);
  emit("press", value, event);
}

function onFocus(): void {
  if (!itemDisabled.value) context.setActiveValue(value);
}

function getNavigationIntent(key: string): ToolbarNavigationIntent | null {
  if (key === "Home") return "first";
  if (key === "End") return "last";
  if (context.orientation.value === "horizontal") {
    const nextKey = context.dir.value === "rtl" ? "ArrowLeft" : "ArrowRight";
    const previousKey = context.dir.value === "rtl" ? "ArrowRight" : "ArrowLeft";
    if (key === nextKey) return "next";
    if (key === previousKey) return "previous";
  } else {
    if (key === "ArrowDown") return "next";
    if (key === "ArrowUp") return "previous";
  }
  return null;
}

function onKeyboard(event: KeyboardEvent, phase: ToolbarKeyboardPhase): void {
  if (phase === "keydown") {
    const intent = getNavigationIntent(event.key);
    if (intent !== null && context.moveFocus(value, intent)) {
      event.preventDefault();
      return;
    }
  }

  if (isNativeButton.value) return;
  const action = getToolbarKeyboardAction(event.key, phase);
  if (action === "ignore") return;
  event.preventDefault();
  if (itemDisabled.value) {
    event.stopImmediatePropagation();
    return;
  }
  if (action === "activate" && event.currentTarget instanceof HTMLElement) {
    event.currentTarget.click();
  }
}

function onKeydown(event: KeyboardEvent): void {
  onKeyboard(event, "keydown");
}

function onKeyup(event: KeyboardEvent): void {
  onKeyboard(event, "keyup");
}

function focusTarget(target: unknown, options?: FocusOptions): void {
  if (
    typeof target === "object" &&
    target !== null &&
    "focus" in target &&
    typeof target.focus === "function"
  ) {
    target.focus(options);
  }
}

function focus(options?: FocusOptions): void {
  focusTarget(element.value, options);
}

type ToolbarItemSetupExpose = Omit<
  ToolbarItemExpose,
  "dir" | "disabled" | "element" | "orientation" | "state"
> & {
  readonly dir: ComputedRef<ToolbarItemSlotState["dir"]>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly orientation: ComputedRef<ToolbarItemSlotState["orientation"]>;
  readonly state: ComputedRef<ToolbarItemState>;
};

const exposed = {
  dir: computed(() => context.dir.value),
  disabled: itemDisabled,
  element,
  focus,
  orientation: computed(() => context.orientation.value),
  state: itemState,
  value,
} satisfies ToolbarItemSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    :type="isNativeButton ? type : undefined"
    :disabled="isNativeButton ? itemDisabled : undefined"
    :role="isNativeButton ? undefined : 'button'"
    :tabindex="tabIndex"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-disabled="itemDisabled && !isNativeButton ? 'true' : undefined"
    data-vize-ui="toolbar-item"
    part="item"
    :data-state="itemState"
    :data-disabled="itemDisabled ? 'true' : undefined"
    :data-orientation="context.orientation.value"
    :data-value="value"
    @click="onClick"
    @focus="onFocus"
    @keydown="onKeydown"
    @keyup="onKeyup"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
