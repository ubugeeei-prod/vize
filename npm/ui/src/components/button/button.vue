<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { getButtonKeyboardAction } from "./keyboard.ts";
import type { ButtonKeyboardPhase } from "./keyboard.ts";
import type { ButtonProps, ButtonSlotProps } from "./types.ts";
import type { PrimitiveElement } from "../../families/foundations/primitive/primitive.ts";

const {
  as = "button",
  native = undefined,
  type = "button",
  disabled = false,
  loading = false,
  variant = "solid",
  tone = "accent",
  size = "md",
} = defineProps<ButtonProps>();

const emit = defineEmits<{
  /**
   * Fired when pointer or keyboard activation reaches the rendered control.
   *
   * Carries the resulting `MouseEvent`.
   */
  press: [event: MouseEvent];
}>();

defineSlots<{
  /** Renders the button contents with current availability state. */
  default(props: ButtonSlotProps): unknown;
}>();

const element = useTemplateRef<PrimitiveElement>("element");
const isNativeButton = computed(() => native ?? as === "button");
const unavailable = computed(() => disabled || loading);
const tabIndex = computed(() => {
  if (isNativeButton.value) return undefined;
  return disabled ? -1 : 0;
});

function onClick(event: MouseEvent): void {
  if (unavailable.value) {
    event.preventDefault();
    event.stopImmediatePropagation();
    return;
  }
  emit("press", event);
}

function onKeyboard(event: KeyboardEvent, phase: ButtonKeyboardPhase): void {
  if (isNativeButton.value) return;
  const action = getButtonKeyboardAction(event.key, phase);
  if (action === "ignore") return;
  event.preventDefault();
  if (unavailable.value) {
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

defineExpose({ element, focus });
</script>

<template>
  <component
    :is="as"
    ref="element"
    :type="isNativeButton ? type : undefined"
    :disabled="isNativeButton ? disabled : undefined"
    :role="isNativeButton ? undefined : 'button'"
    :tabindex="tabIndex"
    :aria-disabled="unavailable && (!isNativeButton || loading) ? 'true' : undefined"
    :aria-busy="loading ? 'true' : undefined"
    data-vize-ui="button"
    :data-state="disabled ? 'disabled' : loading ? 'loading' : 'idle'"
    :data-variant="variant"
    :data-tone="tone"
    :data-size="size"
    @click="onClick"
    @keydown="onKeydown"
    @keyup="onKeyup"
  >
    <slot :disabled :loading :unavailable />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
