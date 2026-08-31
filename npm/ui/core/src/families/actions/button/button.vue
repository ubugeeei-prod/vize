<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { getButtonKeyboardAction } from "./button-keyboard.ts";
import type { ButtonKeyboardPhase } from "./button-keyboard.ts";
import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";

const {
  as = "button",
  native = undefined,
  type = "button",
  disabled = false,
  loading = false,
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
   * Remove the control from activation and sequential keyboard focus.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Announce in-progress work and prevent repeated activation while preserving focus.
   *
   * @default false
   */
  readonly loading?: boolean;
}>();

defineSlots<{
  /** Renders the button contents with current availability state. */
  default(props: {
    readonly disabled: boolean;
    readonly loading: boolean;
    readonly unavailable: boolean;
  }): unknown;
}>();

const emit = defineEmits<{
  /**
   * Fired when pointer or keyboard activation reaches the rendered control.
   *
   * Carries the resulting `MouseEvent`.
   */
  press: [event: MouseEvent];
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
