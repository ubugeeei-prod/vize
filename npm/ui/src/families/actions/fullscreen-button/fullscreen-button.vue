<script setup lang="ts">
import { computed, shallowRef, toRaw, useTemplateRef } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import type {
  FullscreenButtonController,
  FullscreenButtonElement,
  FullscreenButtonEmits,
  FullscreenButtonExpose,
  FullscreenButtonOperation,
  FullscreenButtonOperationType,
  FullscreenButtonProps,
  FullscreenButtonSlotState,
  FullscreenButtonState,
  FullscreenButtonTarget,
  FullscreenButtonType,
} from "./fullscreen-button-types.ts";

type FullscreenButtonKeyboardPhase = "keydown" | "keyup";
type FullscreenButtonKeyboardAction = "activate" | "prevent" | "ignore";

const FULLSCREEN_BUTTON_ACTION_UNAVAILABLE = "VIZE_UI_FULLSCREEN_BUTTON_ACTION_UNAVAILABLE";

const {
  as = "button",
  native = undefined,
  type = "button",
  disabled = false,
  enterLabel = "Enter fullscreen",
  exitLabel = "Exit fullscreen",
  busyLabel = "Changing fullscreen",
  errorLabel = "Fullscreen failed",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  target = undefined,
  controller = undefined,
} = defineProps<FullscreenButtonProps>();

defineSlots<{
  /** Renders button contents with the current fullscreen state. */
  default(props: FullscreenButtonSlotState): unknown;
}>();

const emit = defineEmits<FullscreenButtonEmits>();

const element = useTemplateRef<FullscreenButtonElement>("element");
const stateValue = shallowRef<FullscreenButtonState>("idle");
const activeValue = shallowRef(false);
const operationValue = shallowRef<FullscreenButtonOperation | null>(null);
const isNativeButton = computed(() => native ?? as === "button");
const buttonType = computed<FullscreenButtonType>(() => type);
const disabledValue = computed(() => disabled);
const pendingValue = computed(
  () => stateValue.value === "entering" || stateValue.value === "exiting",
);
const pendingOperation = computed<FullscreenButtonOperationType | null>(
  () => operationValue.value?.type ?? null,
);
const unavailable = computed(() => disabledValue.value || pendingValue.value);
const controllerValue = computed<FullscreenButtonController>(() =>
  toRaw(controller ?? platformFullscreenController),
);
const label = computed(() =>
  pendingValue.value
    ? busyLabel
    : stateValue.value === "error"
      ? errorLabel
      : activeValue.value
        ? exitLabel
        : enterLabel,
);
const tabIndex = computed(() => {
  if (isNativeButton.value) return undefined;
  return disabledValue.value ? -1 : 0;
});
const ariaLabelValue = computed(() => (hasText(ariaLabel) ? ariaLabel : undefined));
const ariaLabelledbyValue = computed(() => (hasText(ariaLabelledby) ? ariaLabelledby : undefined));
const slotState = computed<FullscreenButtonSlotState>(() => ({
  active: activeValue.value,
  disabled: disabledValue.value,
  label: label.value,
  operation: pendingOperation.value,
  pending: pendingValue.value,
  state: stateValue.value,
  unavailable: unavailable.value,
}));

function onClick(event: MouseEvent): void {
  if (unavailable.value) {
    event.preventDefault();
    event.stopImmediatePropagation();
    return;
  }

  const submittedOperation = createFullscreenOperation(controllerValue.value, target, event);
  void runFullscreenOperation(submittedOperation, event);
}

async function runFullscreenOperation(
  submittedOperation: FullscreenButtonOperation,
  event: MouseEvent,
): Promise<void> {
  operationValue.value = submittedOperation;
  stateValue.value = submittedOperation.type === "enter" ? "entering" : "exiting";
  if (submittedOperation.type === "exit") activeValue.value = true;

  try {
    if (submittedOperation.type === "enter") {
      if (submittedOperation.target === null) {
        throw new Error(FULLSCREEN_BUTTON_ACTION_UNAVAILABLE);
      }
      await submittedOperation.controller.requestFullscreen(submittedOperation.target, event);
      activeValue.value = true;
      stateValue.value = "active";
    } else {
      await submittedOperation.controller.exitFullscreen(event);
      activeValue.value = false;
      stateValue.value = "idle";
    }
    emit("fullscreen", submittedOperation, event);
  } catch (error) {
    activeValue.value = submittedOperation.type === "exit";
    stateValue.value = "error";
    emit("error", error, submittedOperation, event);
  } finally {
    if (operationValue.value === submittedOperation) operationValue.value = null;
  }
}

function createFullscreenOperation(
  submittedController: FullscreenButtonController,
  submittedTarget: FullscreenButtonTarget,
  event: MouseEvent,
): FullscreenButtonOperation {
  const activeTarget = getFullscreenElement(submittedController);
  if (activeTarget !== null || activeValue.value) {
    return {
      controller: submittedController,
      target: activeTarget,
      type: "exit",
    };
  }

  return {
    controller: submittedController,
    target: resolveFullscreenTarget(submittedTarget, event),
    type: "enter",
  };
}

function resolveFullscreenTarget(
  submittedTarget: FullscreenButtonTarget,
  event: MouseEvent,
): Element | null {
  if (typeof submittedTarget === "function") {
    return normalizeElement(submittedTarget(event));
  }
  if (submittedTarget !== undefined) return normalizeElement(submittedTarget);
  return getPlatformDocument()?.documentElement ?? null;
}

function getFullscreenElement(submittedController: FullscreenButtonController): Element | null {
  return normalizeElement(submittedController.getFullscreenElement?.());
}

function normalizeElement(value: Element | null | undefined): Element | null {
  if (value == null) return null;
  if (typeof Element === "undefined") return null;
  return value instanceof Element ? value : null;
}

function onKeyboard(event: KeyboardEvent, phase: FullscreenButtonKeyboardPhase): void {
  if (isNativeButton.value) return;
  const keyboardAction = getFullscreenButtonKeyboardAction(event.key, phase);
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

function getFullscreenButtonKeyboardAction(
  key: string,
  phase: FullscreenButtonKeyboardPhase,
): FullscreenButtonKeyboardAction {
  if (key === "Enter") return phase === "keydown" ? "activate" : "ignore";
  if (key === " ") return phase === "keydown" ? "prevent" : "activate";
  return "ignore";
}

function focus(options?: FocusOptions): void {
  focusTarget(element.value, options);
}

function focusTarget(targetElement: unknown, options?: FocusOptions): void {
  if (
    typeof targetElement === "object" &&
    targetElement !== null &&
    "focus" in targetElement &&
    typeof targetElement.focus === "function"
  ) {
    targetElement.focus(options);
  }
}

function getPlatformDocument(): Document | null {
  return typeof globalThis.document === "undefined" ? null : globalThis.document;
}

const platformFullscreenController: FullscreenButtonController = {
  getFullscreenElement() {
    return getPlatformDocument()?.fullscreenElement ?? null;
  },
  requestFullscreen(targetElement) {
    const requestFullscreen = targetElement.requestFullscreen;
    if (typeof requestFullscreen !== "function") {
      throw new Error(FULLSCREEN_BUTTON_ACTION_UNAVAILABLE);
    }
    return requestFullscreen.call(targetElement);
  },
  exitFullscreen() {
    const platformDocument = getPlatformDocument();
    const exitFullscreen = platformDocument?.exitFullscreen;
    if (typeof exitFullscreen !== "function") {
      throw new Error(FULLSCREEN_BUTTON_ACTION_UNAVAILABLE);
    }
    return exitFullscreen.call(platformDocument);
  },
};

function hasText(nextValue: string | undefined): boolean {
  return nextValue != null && nextValue.trim().length > 0;
}

type FullscreenButtonSetupExpose = Omit<
  FullscreenButtonExpose,
  "active" | "disabled" | "element" | "label" | "operation" | "pending" | "state" | "unavailable"
> & {
  readonly active: Readonly<ShallowRef<boolean>>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly label: ComputedRef<string>;
  readonly operation: ComputedRef<FullscreenButtonOperationType | null>;
  readonly pending: ComputedRef<boolean>;
  readonly state: Readonly<ShallowRef<FullscreenButtonState>>;
  readonly unavailable: ComputedRef<boolean>;
};

const exposed = {
  active: activeValue,
  disabled: disabledValue,
  element,
  focus,
  label,
  operation: pendingOperation,
  pending: pendingValue,
  state: stateValue,
  unavailable,
} satisfies FullscreenButtonSetupExpose;

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
    :aria-pressed="activeValue ? 'true' : 'false'"
    :aria-disabled="unavailable && (!isNativeButton || pendingValue) ? 'true' : undefined"
    :aria-busy="pendingValue ? 'true' : undefined"
    data-vize-ui="fullscreen-button"
    part="root"
    :data-state="stateValue"
    :data-disabled="disabledValue ? 'true' : undefined"
    :data-active="activeValue ? 'true' : undefined"
    :data-pending="pendingValue ? 'true' : undefined"
    @click="onClick"
    @keydown="onKeydown"
    @keyup="onKeyup"
  >
    <slot v-bind="slotState">
      <span data-vize-ui="fullscreen-button-label" part="label">{{ label }}</span>
    </slot>
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
