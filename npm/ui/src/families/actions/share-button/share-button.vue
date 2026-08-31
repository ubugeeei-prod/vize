<script setup lang="ts">
import { computed, shallowRef, toRaw, useTemplateRef } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import type {
  ShareButtonAction,
  ShareButtonElement,
  ShareButtonEmits,
  ShareButtonExpose,
  ShareButtonPayload,
  ShareButtonProps,
  ShareButtonSlotState,
  ShareButtonState,
  ShareButtonType,
} from "./share-button-types.ts";

type ShareButtonKeyboardPhase = "keydown" | "keyup";
type ShareButtonKeyboardAction = "activate" | "prevent" | "ignore";

type MutableShareButtonPayload = {
  title?: string;
  text?: string;
  url?: string;
  files?: File[];
};

type ShareButtonPayloadSource = {
  readonly title: string | undefined;
  readonly text: string | undefined;
  readonly url: string | undefined;
  readonly files: File[] | undefined;
};

type PlatformNavigatorWithShare = {
  readonly share?: (payload: ShareButtonPayload) => void | Promise<void>;
};

const SHARE_BUTTON_ACTION_UNAVAILABLE = "VIZE_UI_SHARE_BUTTON_ACTION_UNAVAILABLE";

const {
  as = "button",
  native = undefined,
  type = "button",
  disabled = false,
  title = undefined,
  text = undefined,
  url = undefined,
  files = undefined,
  idleLabel = "Share",
  sharingLabel = "Sharing",
  sharedLabel = "Shared",
  errorLabel = "Share failed",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  action = undefined,
} = defineProps<ShareButtonProps>();

defineSlots<{
  /** Renders button contents with the current share state. */
  default(props: ShareButtonSlotState): unknown;
}>();

const emit = defineEmits<ShareButtonEmits>();

const element = useTemplateRef<ShareButtonElement>("element");
const stateValue = shallowRef<ShareButtonState>("idle");
const isNativeButton = computed(() => native ?? as === "button");
const buttonType = computed<ShareButtonType>(() => type);
const disabledValue = computed(() => disabled);
const sharingValue = computed(() => stateValue.value === "sharing");
const unavailable = computed(() => disabledValue.value || sharingValue.value);
const actionValue = computed<ShareButtonAction>(() => toRaw(action ?? shareWithNavigator));
const payloadValue = computed<ShareButtonPayload>(() =>
  normalizeSharePayload({ files, text, title, url }),
);
const label = computed(() =>
  stateValue.value === "sharing"
    ? sharingLabel
    : stateValue.value === "shared"
      ? sharedLabel
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
const slotState = computed<ShareButtonSlotState>(() => ({
  disabled: disabledValue.value,
  label: label.value,
  payload: payloadValue.value,
  sharing: sharingValue.value,
  state: stateValue.value,
  unavailable: unavailable.value,
}));

function onClick(event: MouseEvent): void {
  if (unavailable.value) {
    event.preventDefault();
    event.stopImmediatePropagation();
    return;
  }

  void runShareAction(actionValue.value, payloadValue.value, event);
}

async function runShareAction(
  submittedAction: ShareButtonAction,
  submittedPayload: ShareButtonPayload,
  event: MouseEvent,
): Promise<void> {
  stateValue.value = "sharing";
  try {
    await submittedAction(submittedPayload, event);
    stateValue.value = "shared";
    emit("share", submittedPayload, event);
  } catch (error) {
    stateValue.value = "error";
    emit("error", error, submittedPayload, event);
  }
}

function onKeyboard(event: KeyboardEvent, phase: ShareButtonKeyboardPhase): void {
  if (isNativeButton.value) return;
  const keyboardAction = getShareButtonKeyboardAction(event.key, phase);
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

function getShareButtonKeyboardAction(
  key: string,
  phase: ShareButtonKeyboardPhase,
): ShareButtonKeyboardAction {
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

function normalizeSharePayload(source: ShareButtonPayloadSource): ShareButtonPayload {
  const payload: MutableShareButtonPayload = {};
  if (source.title !== undefined) payload.title = source.title;
  if (source.text !== undefined) payload.text = source.text;
  if (source.url !== undefined) payload.url = source.url;
  if (source.files !== undefined) payload.files = [...source.files];
  return payload;
}

async function shareWithNavigator(payload: ShareButtonPayload): Promise<void> {
  const platformNavigator = globalThis.navigator as PlatformNavigatorWithShare | undefined;
  const share = platformNavigator?.share;
  if (typeof share !== "function") throw new Error(SHARE_BUTTON_ACTION_UNAVAILABLE);
  await share.call(platformNavigator, payload);
}

function hasText(nextValue: string | undefined): boolean {
  return nextValue != null && nextValue.trim().length > 0;
}

type ShareButtonSetupExpose = Omit<
  ShareButtonExpose,
  "disabled" | "element" | "label" | "payload" | "sharing" | "state" | "unavailable"
> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly label: ComputedRef<string>;
  readonly payload: ComputedRef<ShareButtonPayload>;
  readonly sharing: ComputedRef<boolean>;
  readonly state: Readonly<ShallowRef<ShareButtonState>>;
  readonly unavailable: ComputedRef<boolean>;
};

const exposed = {
  disabled: disabledValue,
  element,
  focus,
  label,
  payload: payloadValue,
  sharing: sharingValue,
  state: stateValue,
  unavailable,
} satisfies ShareButtonSetupExpose;

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
    :aria-disabled="unavailable && (!isNativeButton || sharingValue) ? 'true' : undefined"
    :aria-busy="sharingValue ? 'true' : undefined"
    data-vize-ui="share-button"
    part="root"
    :data-state="stateValue"
    :data-disabled="disabledValue ? 'true' : undefined"
    :data-sharing="sharingValue ? 'true' : undefined"
    @click="onClick"
    @keydown="onKeydown"
    @keyup="onKeyup"
  >
    <slot v-bind="slotState">
      <span data-vize-ui="share-button-label" part="label">{{ label }}</span>
    </slot>
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
