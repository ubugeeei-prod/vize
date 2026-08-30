<script setup lang="ts">
import { computed, onUnmounted, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import type { PrimitiveAs, PrimitiveElement } from "../../../primitive.ts";
import { buttonGroupContext } from "./button-group-context.ts";
import type { ButtonGroupNavigationIntent } from "./button-group-context.ts";
import type {
  ButtonGroupItemExpose,
  ButtonGroupItemSlotState,
  ButtonGroupItemState,
} from "./button-group-types.ts";

type ButtonGroupKeyboardPhase = "keydown" | "keyup";
type ButtonGroupKeyboardAction = "activate" | "prevent" | "ignore";

function getButtonGroupKeyboardAction(
  key: string,
  phase: ButtonGroupKeyboardPhase,
): ButtonGroupKeyboardAction {
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
} = defineProps<{
  /** Native element, custom element, or component to render. @default "button" */
  readonly as?: PrimitiveAs;
  /** Whether the rendered target already implements native button semantics. @default auto */
  readonly native?: boolean;
  /** Native button submission behavior. @default "button" */
  readonly type?: "button" | "reset" | "submit";
  /** Stable item value emitted by item and group press events. @default required */
  readonly value: string;
  /** Disable this item while preserving the rest of the group. @default false */
  readonly disabled?: boolean;
  /** Accessible name when no visible label or `aria-labelledby` supplies one. @default undefined */
  readonly ariaLabel?: string;
  /** Space-separated ids that label this item. @default undefined */
  readonly ariaLabelledby?: string;
  /** Space-separated ids that describe this item. @default undefined */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** Renders item contents with current availability and navigation state. */
  default(props: ButtonGroupItemSlotState): unknown;
}>();

const emit = defineEmits<{
  /** Fired after user activation reaches this enabled item. */
  press: [value: string, nativeEvent: MouseEvent];
}>();

const context = buttonGroupContext.use();
const element = useTemplateRef<PrimitiveElement>("element");
const isNativeButton = computed(() => native ?? as === "button");
const itemDisabled = computed(() => context.disabled.value || disabled);
const itemState = computed<ButtonGroupItemState>(() => context.getItemState(itemDisabled.value));
const tabIndex = computed(() => {
  if (itemDisabled.value) return isNativeButton.value ? undefined : -1;
  if (!context.rovingFocus.value) return isNativeButton.value ? undefined : 0;
  return context.activeValue.value === value ? 0 : -1;
});
const slotState = computed<ButtonGroupItemSlotState>(() => ({
  disabled: itemDisabled.value,
  orientation: context.orientation.value,
  state: itemState.value,
  value,
}));

const unregister = context.registerItem({
  disabled: itemDisabled,
  element,
  value: () => value,
});

watch([itemDisabled, () => value], () => context.syncActiveValue(), { flush: "post" });
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

function getNavigationIntent(key: string): ButtonGroupNavigationIntent | null {
  if (key === "Home") return "first";
  if (key === "End") return "last";
  if (context.orientation.value === "horizontal") {
    if (key === "ArrowRight") return "next";
    if (key === "ArrowLeft") return "previous";
  } else {
    if (key === "ArrowDown") return "next";
    if (key === "ArrowUp") return "previous";
  }
  return null;
}

function onKeyboard(event: KeyboardEvent, phase: ButtonGroupKeyboardPhase): void {
  if (phase === "keydown") {
    const intent = getNavigationIntent(event.key);
    if (intent !== null && context.moveFocus(value, intent)) {
      event.preventDefault();
      return;
    }
  }

  if (isNativeButton.value) return;
  const action = getButtonGroupKeyboardAction(event.key, phase);
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

type ButtonGroupItemSetupExpose = Omit<
  ButtonGroupItemExpose,
  "disabled" | "element" | "orientation" | "state"
> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly orientation: ComputedRef<ButtonGroupItemSlotState["orientation"]>;
  readonly state: ComputedRef<ButtonGroupItemState>;
};

const exposed = {
  disabled: itemDisabled,
  element,
  focus,
  orientation: computed(() => context.orientation.value),
  state: itemState,
  value,
} satisfies ButtonGroupItemSetupExpose;

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
    data-vize-ui="button-group-item"
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
