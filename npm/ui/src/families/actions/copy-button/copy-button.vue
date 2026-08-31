<script setup lang="ts">
import { computed, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import type {
  CopyButtonElement,
  CopyButtonEmits,
  CopyButtonExpose,
  CopyButtonProps,
  CopyButtonSlotState,
  CopyButtonState,
  CopyButtonType,
} from "./copy-button-types.ts";

type CopyButtonKeyboardPhase = "keydown" | "keyup";
type CopyButtonKeyboardAction = "activate" | "prevent" | "ignore";

const COPY_BUTTON_CLIPBOARD_UNAVAILABLE = "VIZE_UI_COPY_BUTTON_CLIPBOARD_UNAVAILABLE";

const {
  as = "button",
  native = undefined,
  type = "button",
  value,
  disabled = false,
  idleLabel = "Copy",
  copiedLabel = "Copied",
  errorLabel = "Copy failed",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  writer = undefined,
} = defineProps<CopyButtonProps>();

defineSlots<{
  /** Renders button contents with the current copy state. */
  default(props: CopyButtonSlotState): unknown;
}>();

const emit = defineEmits<CopyButtonEmits>();

const element = useTemplateRef<CopyButtonElement>("element");
const stateValue = shallowRef<CopyButtonState>("idle");
const writingValue = shallowRef(false);
const isNativeButton = computed(() => native ?? as === "button");
const buttonType = computed<CopyButtonType>(() => type);
const disabledValue = computed(() => disabled);
const copyValue = computed(() => value);
const unavailable = computed(() => disabledValue.value || writingValue.value);
const label = computed(() =>
  stateValue.value === "copied"
    ? copiedLabel
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
const slotState = computed<CopyButtonSlotState>(() => ({
  disabled: disabledValue.value,
  label: label.value,
  state: stateValue.value,
  unavailable: unavailable.value,
  value,
  writing: writingValue.value,
}));

watch(
  () => value,
  () => {
    stateValue.value = "idle";
  },
  { flush: "sync" },
);

function onClick(event: MouseEvent): void {
  if (unavailable.value) {
    event.preventDefault();
    event.stopImmediatePropagation();
    return;
  }

  void copyValueToClipboard(copyValue.value, event);
}

async function copyValueToClipboard(nextValue: string, event: MouseEvent): Promise<void> {
  writingValue.value = true;
  try {
    await (writer ?? writeTextWithNavigatorClipboard)(nextValue);
    stateValue.value = "copied";
    emit("copy", nextValue, event);
  } catch (error) {
    stateValue.value = "error";
    emit("error", error, nextValue, event);
  } finally {
    writingValue.value = false;
  }
}

function onKeyboard(event: KeyboardEvent, phase: CopyButtonKeyboardPhase): void {
  if (isNativeButton.value) return;
  const action = getCopyButtonKeyboardAction(event.key, phase);
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

function getCopyButtonKeyboardAction(
  key: string,
  phase: CopyButtonKeyboardPhase,
): CopyButtonKeyboardAction {
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

async function writeTextWithNavigatorClipboard(nextValue: string): Promise<void> {
  const writeText = globalThis.navigator?.clipboard?.writeText;
  if (typeof writeText !== "function") throw new Error(COPY_BUTTON_CLIPBOARD_UNAVAILABLE);
  await writeText.call(globalThis.navigator.clipboard, nextValue);
}

function hasText(nextValue: string | undefined): boolean {
  return nextValue != null && nextValue.trim().length > 0;
}

type CopyButtonSetupExpose = Omit<
  CopyButtonExpose,
  "disabled" | "element" | "label" | "state" | "unavailable" | "value" | "writing"
> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly label: ComputedRef<string>;
  readonly state: Readonly<ShallowRef<CopyButtonState>>;
  readonly unavailable: ComputedRef<boolean>;
  readonly value: ComputedRef<string>;
  readonly writing: Readonly<ShallowRef<boolean>>;
};

const exposed = {
  disabled: disabledValue,
  element,
  focus,
  label,
  state: stateValue,
  unavailable,
  value: copyValue,
  writing: writingValue,
} satisfies CopyButtonSetupExpose;

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
    :aria-disabled="unavailable && (!isNativeButton || writingValue) ? 'true' : undefined"
    :aria-busy="writingValue ? 'true' : undefined"
    data-vize-ui="copy-button"
    part="root"
    :data-state="stateValue"
    :data-disabled="disabledValue ? 'true' : undefined"
    :data-writing="writingValue ? 'true' : undefined"
    @click="onClick"
    @keydown="onKeydown"
    @keyup="onKeyup"
  >
    <slot v-bind="slotState">
      <span data-vize-ui="copy-button-label" part="label">{{ label }}</span>
    </slot>
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
