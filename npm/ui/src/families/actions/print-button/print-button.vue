<script setup lang="ts">
import { computed, shallowRef, useTemplateRef } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import type {
  PrintButtonAction,
  PrintButtonElement,
  PrintButtonEmits,
  PrintButtonExpose,
  PrintButtonProps,
  PrintButtonSlotState,
  PrintButtonState,
  PrintButtonType,
} from "./print-button-types.ts";

type PrintButtonKeyboardPhase = "keydown" | "keyup";
type PrintButtonKeyboardAction = "activate" | "prevent" | "ignore";

const PRINT_BUTTON_ACTION_UNAVAILABLE = "VIZE_UI_PRINT_BUTTON_ACTION_UNAVAILABLE";

const {
  as = "button",
  native = undefined,
  type = "button",
  disabled = false,
  idleLabel = "Print",
  printingLabel = "Printing",
  printedLabel = "Printed",
  errorLabel = "Print failed",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  action = undefined,
} = defineProps<PrintButtonProps>();

defineSlots<{
  /** Renders button contents with the current print state. */
  default(props: PrintButtonSlotState): unknown;
}>();

const emit = defineEmits<PrintButtonEmits>();

const element = useTemplateRef<PrintButtonElement>("element");
const stateValue = shallowRef<PrintButtonState>("idle");
const isNativeButton = computed(() => native ?? as === "button");
const buttonType = computed<PrintButtonType>(() => type);
const disabledValue = computed(() => disabled);
const printingValue = computed(() => stateValue.value === "printing");
const unavailable = computed(() => disabledValue.value || printingValue.value);
const actionValue = computed<PrintButtonAction>(() => action ?? printWithGlobal);
const label = computed(() =>
  stateValue.value === "printing"
    ? printingLabel
    : stateValue.value === "printed"
      ? printedLabel
      : stateValue.value === "error"
        ? errorLabel
        : idleLabel,
);
const tabIndex = computed(() => {
  if (isNativeButton.value) return undefined;
  return disabledValue.value ? -1 : 0;
});
const ariaLabelValue = computed(() => (hasText(ariaLabel) ? ariaLabel : undefined));
const ariaLabelledbyValue = computed(() => (hasText(ariaLabelledby) ? ariaLabelledby : undefined));
const slotState = computed<PrintButtonSlotState>(() => ({
  disabled: disabledValue.value,
  label: label.value,
  printing: printingValue.value,
  state: stateValue.value,
  unavailable: unavailable.value,
}));

function onClick(event: MouseEvent): void {
  if (unavailable.value) {
    event.preventDefault();
    event.stopImmediatePropagation();
    return;
  }

  void runPrintAction(actionValue.value, event);
}

async function runPrintAction(
  submittedAction: PrintButtonAction,
  event: MouseEvent,
): Promise<void> {
  stateValue.value = "printing";
  try {
    await submittedAction(event);
    stateValue.value = "printed";
    emit("print", event);
  } catch (error) {
    stateValue.value = "error";
    emit("error", error, event);
  }
}

function onKeyboard(event: KeyboardEvent, phase: PrintButtonKeyboardPhase): void {
  if (isNativeButton.value) return;
  const keyboardAction = getPrintButtonKeyboardAction(event.key, phase);
  if (keyboardAction === "ignore") return;

  event.preventDefault();
  if (unavailable.value) {
    event.stopImmediatePropagation();
    return;
  }
  if (keyboardAction === "activate" && event.currentTarget instanceof HTMLElement) {
    event.currentTarget.click();
  }
}

function onKeydown(event: KeyboardEvent): void {
  onKeyboard(event, "keydown");
}

function onKeyup(event: KeyboardEvent): void {
  onKeyboard(event, "keyup");
}

function getPrintButtonKeyboardAction(
  key: string,
  phase: PrintButtonKeyboardPhase,
): PrintButtonKeyboardAction {
  if (key === "Enter") return phase === "keydown" ? "activate" : "ignore";
  if (key === " ") return phase === "keydown" ? "prevent" : "activate";
  return "ignore";
}

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

function printWithGlobal(): void {
  const print = globalThis.print;
  if (typeof print !== "function") throw new Error(PRINT_BUTTON_ACTION_UNAVAILABLE);
  print.call(globalThis);
}

function hasText(nextValue: string | undefined): boolean {
  return nextValue != null && nextValue.trim().length > 0;
}

type PrintButtonSetupExpose = Omit<
  PrintButtonExpose,
  "disabled" | "element" | "label" | "printing" | "state" | "unavailable"
> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly label: ComputedRef<string>;
  readonly printing: ComputedRef<boolean>;
  readonly state: Readonly<ShallowRef<PrintButtonState>>;
  readonly unavailable: ComputedRef<boolean>;
};

const exposed = {
  disabled: disabledValue,
  element,
  focus,
  label,
  printing: printingValue,
  state: stateValue,
  unavailable,
} satisfies PrintButtonSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    :type="isNativeButton ? buttonType : undefined"
    :disabled="isNativeButton ? disabledValue : undefined"
    :role="isNativeButton ? undefined : 'button'"
    :tabindex="tabIndex"
    :aria-label="ariaLabelledbyValue === undefined ? ariaLabelValue : undefined"
    :aria-labelledby="ariaLabelledbyValue"
    :aria-describedby="ariaDescribedby"
    :aria-disabled="unavailable && (!isNativeButton || printingValue) ? 'true' : undefined"
    :aria-busy="printingValue ? 'true' : undefined"
    data-vize-ui="print-button"
    part="root"
    :data-state="stateValue"
    :data-disabled="disabledValue ? 'true' : undefined"
    :data-printing="printingValue ? 'true' : undefined"
    @click="onClick"
    @keydown="onKeydown"
    @keyup="onKeyup"
  >
    <slot v-bind="slotState">
      <span data-vize-ui="print-button-label" part="label">{{ label }}</span>
    </slot>
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
