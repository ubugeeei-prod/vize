<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type {
  IconButtonElement,
  IconButtonExpose,
  IconButtonProps,
  IconButtonSize,
  IconButtonSlotState,
  IconButtonState,
  IconButtonTone,
  IconButtonVariant,
} from "./icon-types.ts";

type IconButtonKeyboardPhase = "keydown" | "keyup";
type IconButtonKeyboardAction = "activate" | "prevent" | "ignore";

const {
  as = "button",
  native = undefined,
  type = "button",
  disabled = false,
  loading = false,
  size = "md",
  tone = "neutral",
  variant = "plain",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<IconButtonProps>();

defineSlots<{
  /** Renders the icon-only contents with current availability and styling hooks. */
  default(props: IconButtonSlotState): unknown;
}>();

const emit = defineEmits<{
  /**
   * Fired when pointer or keyboard activation reaches the rendered control.
   *
   * Carries the resulting `MouseEvent`.
   */
  press: [event: MouseEvent];
}>();

const element = useTemplateRef<IconButtonElement>("element");
const isNativeButton = computed(() => native ?? as === "button");
const disabledValue = computed(() => disabled);
const loadingValue = computed(() => loading);
const unavailable = computed(() => disabledValue.value || loadingValue.value);
const sizeValue = computed<IconButtonSize>(() => size);
const toneValue = computed<IconButtonTone>(() => tone);
const variantValue = computed<IconButtonVariant>(() => variant);
const state = computed<IconButtonState>(() =>
  disabledValue.value ? "disabled" : loadingValue.value ? "loading" : "idle",
);
const tabIndex = computed(() => {
  if (isNativeButton.value) return undefined;
  return disabledValue.value ? -1 : 0;
});
const ariaLabelValue = computed(() => (hasText(ariaLabel) ? ariaLabel : undefined));
const ariaLabelledbyValue = computed(() => (hasText(ariaLabelledby) ? ariaLabelledby : undefined));
const slotState = computed<IconButtonSlotState>(() => ({
  disabled: disabledValue.value,
  loading: loadingValue.value,
  size: sizeValue.value,
  state: state.value,
  tone: toneValue.value,
  unavailable: unavailable.value,
  variant: variantValue.value,
}));

function onClick(event: MouseEvent): void {
  if (unavailable.value) {
    event.preventDefault();
    event.stopImmediatePropagation();
    return;
  }
  emit("press", event);
}

function onKeyboard(event: KeyboardEvent, phase: IconButtonKeyboardPhase): void {
  if (isNativeButton.value) return;
  const action = getIconButtonKeyboardAction(event.key, phase);
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

function getIconButtonKeyboardAction(
  key: string,
  phase: IconButtonKeyboardPhase,
): IconButtonKeyboardAction {
  if (key === "Enter") return phase === "keydown" ? "activate" : "ignore";
  if (key === " ") return phase === "keydown" ? "prevent" : "activate";
  return "ignore";
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

type IconButtonSetupExpose = Omit<
  IconButtonExpose,
  "disabled" | "element" | "loading" | "size" | "state" | "tone" | "unavailable" | "variant"
> & {
  readonly disabled: typeof disabledValue;
  readonly element: typeof element;
  readonly loading: typeof loadingValue;
  readonly size: typeof sizeValue;
  readonly state: typeof state;
  readonly tone: typeof toneValue;
  readonly unavailable: typeof unavailable;
  readonly variant: typeof variantValue;
};

const exposed = {
  disabled: disabledValue,
  element,
  focus,
  loading: loadingValue,
  size: sizeValue,
  state,
  tone: toneValue,
  unavailable,
  variant: variantValue,
} satisfies IconButtonSetupExpose;

defineExpose(exposed);

function hasText(value: string | undefined): boolean {
  return value != null && value.trim().length > 0;
}
</script>

<template>
  <component
    :is="as"
    ref="element"
    :type="isNativeButton ? type : undefined"
    :disabled="isNativeButton ? disabledValue : undefined"
    :role="isNativeButton ? undefined : 'button'"
    :tabindex="tabIndex"
    :aria-disabled="unavailable && (!isNativeButton || loadingValue) ? 'true' : undefined"
    :aria-busy="loadingValue ? 'true' : undefined"
    :aria-label="ariaLabelledbyValue === undefined ? ariaLabelValue : undefined"
    :aria-labelledby="ariaLabelledbyValue"
    :aria-describedby="ariaDescribedby"
    data-vize-ui="icon-button"
    part="root"
    :data-state="state"
    :data-size="sizeValue"
    :data-tone="toneValue"
    :data-variant="variantValue"
    :data-name="
      ariaLabelValue !== undefined || ariaLabelledbyValue !== undefined ? 'present' : 'missing'
    "
    @click="onClick"
    @keydown="onKeydown"
    @keyup="onKeyup"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
