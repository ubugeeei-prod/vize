<script setup lang="ts">
import { computed, onUnmounted, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { toggleGroupContext } from "./toggle-group-context.ts";
import type { ToggleGroupNavigationIntent } from "./toggle-group-context.ts";
import type {
  ToggleGroupItemExpose,
  ToggleGroupItemSlotState,
  ToggleGroupItemState,
} from "./toggle-group-types.ts";
import type { PrimitiveAs, PrimitiveElement } from "./primitive.ts";

type ToggleGroupKeyboardPhase = "keydown" | "keyup";
type ToggleGroupKeyboardAction = "activate" | "prevent" | "ignore";

function getToggleGroupKeyboardAction(
  key: string,
  phase: ToggleGroupKeyboardPhase,
): ToggleGroupKeyboardAction {
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
  /** Item value used by the group selection model. @default required */
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
  /** Renders item contents with current pressed and availability state. */
  default(props: ToggleGroupItemSlotState): unknown;
}>();

const emit = defineEmits<{
  /** Fired after user activation requests this item to toggle. */
  press: [value: string, pressed: boolean, nativeEvent: MouseEvent];
}>();

const context = toggleGroupContext.use();
const element = useTemplateRef<PrimitiveElement>("element");
const isNativeButton = computed(() => native ?? as === "button");
const pressed = computed(() => context.isPressed(value));
const itemDisabled = computed(() => context.disabled.value || disabled);
const itemState = computed<ToggleGroupItemState>(() =>
  context.getItemState(value, itemDisabled.value),
);
const tabIndex = computed(() => {
  if (itemDisabled.value) return isNativeButton.value ? undefined : -1;
  if (!context.rovingFocus.value) return isNativeButton.value ? undefined : 0;
  return context.activeValue.value === value ? 0 : -1;
});
const slotState = computed<ToggleGroupItemSlotState>(() => ({
  disabled: itemDisabled.value,
  orientation: context.orientation.value,
  pressed: pressed.value,
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

  const nextPressed = !pressed.value;
  const changed = context.requestItemToggle(value, event);
  if (changed) emit("press", value, nextPressed, event);
}

function onFocus(): void {
  if (!itemDisabled.value) context.setActiveValue(value);
}

function getNavigationIntent(key: string): ToggleGroupNavigationIntent | null {
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

function onKeyboard(event: KeyboardEvent, phase: ToggleGroupKeyboardPhase): void {
  if (phase === "keydown") {
    const intent = getNavigationIntent(event.key);
    if (intent !== null) {
      event.preventDefault();
      context.moveFocus(value, intent);
      return;
    }
  }

  if (isNativeButton.value) return;
  const action = getToggleGroupKeyboardAction(event.key, phase);
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

type ToggleGroupItemSetupExpose = Omit<
  ToggleGroupItemExpose,
  "disabled" | "element" | "orientation" | "pressed" | "state"
> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly orientation: ComputedRef<ToggleGroupItemSlotState["orientation"]>;
  readonly pressed: ComputedRef<boolean>;
  readonly state: ComputedRef<ToggleGroupItemState>;
};

const exposed = {
  disabled: itemDisabled,
  element,
  focus,
  orientation: computed(() => context.orientation.value),
  pressed,
  state: itemState,
  value,
} satisfies ToggleGroupItemSetupExpose;

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
    :aria-pressed="pressed ? 'true' : 'false'"
    data-vize-ui="toggle-group-item"
    part="item"
    :data-state="itemState"
    :data-pressed="pressed ? 'true' : 'false'"
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
