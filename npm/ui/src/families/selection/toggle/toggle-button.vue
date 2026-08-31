<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";
import type { ToggleExpose, ToggleSlotState } from "./toggle-types.ts";

type ToggleKeyboardPhase = "keydown" | "keyup";
type ToggleKeyboardAction = "activate" | "prevent" | "ignore";

function getToggleKeyboardAction(key: string, phase: ToggleKeyboardPhase): ToggleKeyboardAction {
  if (key === "Enter") return phase === "keydown" ? "activate" : "ignore";
  if (key === " ") return phase === "keydown" ? "prevent" : "activate";
  return "ignore";
}

const {
  as = "button",
  native = undefined,
  type = "button",
  modelValue = undefined,
  defaultPressed = false,
  disabled = false,
  ariaLabel,
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "button"
   */
  readonly as?: PrimitiveAs;

  /**
   * Whether the rendered target already implements native button semantics.
   *
   * @default true when `as` is "button"; otherwise false
   */
  readonly native?: boolean;

  /**
   * Native button submission behavior.
   *
   * @default "button"
   */
  readonly type?: "button" | "reset" | "submit";

  /**
   * Controlled pressed value. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: boolean;

  /**
   * Initial pressed state for uncontrolled use.
   *
   * @default false
   */
  readonly defaultPressed?: boolean;

  /**
   * Remove the control from activation and sequential keyboard focus.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Accessible name when no visible label supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Renders the toggle contents with current pressed and availability state. */
  default(props: ToggleSlotState): unknown;
}>();

const emit = defineEmits<{
  /** Fired when the pressed state requests a new controlled boolean value. */
  "update:modelValue": [value: boolean];
  /** Fired after user interaction requests the next pressed value. */
  change: [value: boolean, nativeEvent: MouseEvent];
}>();

const element = useTemplateRef<PrimitiveElement>("element");
const isNativeButton = computed(() => native ?? as === "button");
const state = useControllableState({
  value: () => modelValue,
  defaultValue: () => defaultPressed,
  onChange: (value) => emit("update:modelValue", value),
});
const pressed = state.value;
const tabIndex = computed(() => {
  if (isNativeButton.value) return undefined;
  return disabled ? -1 : 0;
});

function onClick(event: MouseEvent): void {
  if (disabled) {
    event.preventDefault();
    event.stopImmediatePropagation();
    return;
  }
  const next = !pressed.value;
  state.set(next);
  emit("change", next, event);
}

function onKeyboard(event: KeyboardEvent, phase: ToggleKeyboardPhase): void {
  if (isNativeButton.value) return;
  const action = getToggleKeyboardAction(event.key, phase);
  if (action === "ignore") return;
  event.preventDefault();
  if (disabled) {
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

/** Move focus to the rendered control when it exposes a focus method. */
function focus(options?: FocusOptions): void {
  focusTarget(element.value, options);
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

const exposed = {
  element,
  focus,
  pressed,
  reset: state.reset,
  setPressed: state.set,
} satisfies ToggleExpose & { readonly element: typeof element };

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    :type="isNativeButton ? type : undefined"
    :disabled="isNativeButton ? disabled : undefined"
    :role="isNativeButton ? undefined : 'button'"
    :tabindex="tabIndex"
    :aria-label="ariaLabel"
    :aria-disabled="disabled && !isNativeButton ? 'true' : undefined"
    :aria-pressed="pressed ? 'true' : 'false'"
    data-vize-ui="toggle"
    :data-state="disabled ? 'disabled' : pressed ? 'pressed' : 'unpressed'"
    @click="onClick"
    @keydown="onKeydown"
    @keyup="onKeyup"
  >
    <slot :disabled :pressed />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
