<script setup lang="ts" generic="Strength">
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { passwordFieldContext } from "./password-field-context.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import type {
  PasswordFieldAriaInvalid,
  PasswordFieldExpose,
  PasswordFieldSlotState,
  PasswordFieldState,
} from "./password-field-types.ts";

const {
  id = undefined,
  name = undefined,
  modelValue = undefined,
  defaultValue = "",
  visible = undefined,
  defaultVisible = false,
  evaluateStrength = undefined,
  autocomplete = "current-password",
  disabled = false,
  readOnly = false,
  required = false,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  ariaInvalid = false,
} = defineProps<{
  /**
   * Id of the password input. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Native form field name of the password input.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Controlled password. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: string;

  /**
   * Initial uncontrolled password, also restored by form reset.
   *
   * @default ""
   */
  readonly defaultValue?: string;

  /**
   * Controlled visibility (`v-model:visible`). `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly visible?: boolean;

  /**
   * Initial uncontrolled visibility.
   *
   * @default false
   */
  readonly defaultVisible?: boolean;

  /**
   * Strength meter hook; its return type becomes the slot's `strength`.
   * Pass `estimatePasswordStrength` or any custom evaluator.
   *
   * @default undefined
   */
  readonly evaluateStrength?: (value: string) => Strength;

  /**
   * Native autocomplete token: `"current-password"` for sign-in, `"new-password"` for sign-up.
   *
   * @default "current-password"
   */
  readonly autocomplete?: string;

  /**
   * Disable editing, the toggle, and native form submission.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Keep the input focusable while preventing edits.
   *
   * @default false
   */
  readonly readOnly?: boolean;

  /**
   * Mark the input as required for native constraint validation.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Accessible name of the input when no label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the input.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the input.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;

  /**
   * Id of the validation error message used while invalid.
   *
   * @default undefined
   */
  readonly ariaErrormessage?: string;

  /**
   * Invalid state announced to assistive technology.
   *
   * @default false
   */
  readonly ariaInvalid?: PasswordFieldAriaInvalid;
}>();

const emit = defineEmits<{
  /** Fired when the password requests a new controlled value. */
  "update:modelValue": [value: string];

  /** Fired when visibility requests a new controlled value (`v-model:visible`). */
  "update:visible": [visible: boolean];

  /** Fired when Caps Lock turns on or off while typing in the input. */
  capsLockChange: [active: boolean];
}>();

defineSlots<{
  /** Renders the input, toggle, and any hints with visibility, Caps Lock, and strength state. */
  default(props: PasswordFieldSlotState<Strength>): unknown;
}>();

const root = useTemplateRef<HTMLDivElement>("root");
const inputElement = shallowRef<HTMLInputElement | null>(null);
const inputId = useDeterministicId({ id: () => id, hint: "password" });
const valueState = useControllableState({
  value: () => modelValue,
  defaultValue: () => defaultValue,
  onChange: (value) => emit("update:modelValue", value),
});
const visibleState = useControllableState({
  value: () => visible,
  defaultValue: () => defaultVisible,
  onChange: (value) => emit("update:visible", value),
});
const capsLock = shallowRef(false);
const ariaInvalidValue = computed(() => {
  if (ariaInvalid === false) return undefined;
  return ariaInvalid === true ? "true" : ariaInvalid;
});
const strength = computed(() => evaluateStrength?.(valueState.value.value));
const dataState = computed<PasswordFieldState>(() => {
  if (disabled) return "disabled";
  if (readOnly) return "readonly";
  return visibleState.value.value ? "visible" : "hidden";
});

function setVisible(next: boolean): boolean {
  if (disabled) return false;
  return visibleState.set(next);
}

function setCapsLock(active: boolean): void {
  if (capsLock.value === active) return;
  capsLock.value = active;
  emit("capsLockChange", active);
}

watch(
  inputElement,
  (input, _previous, onCleanup) => {
    const owner = input?.form;
    if (owner === undefined || owner === null) return;
    const onReset = () => {
      if (!valueState.controlled.value) valueState.reset();
      if (!visibleState.controlled.value) visibleState.reset();
      void nextTick(() => {
        if (input !== null) input.value = valueState.value.value;
      });
    };
    owner.addEventListener("reset", onReset);
    onCleanup(() => owner.removeEventListener("reset", onReset));
  },
  { flush: "post" },
);

passwordFieldContext.provide({
  inputId,
  name: computed(() => name),
  value: valueState.value,
  visible: visibleState.value,
  capsLock: computed(() => capsLock.value),
  disabled: computed(() => disabled),
  readOnly: computed(() => readOnly),
  required: computed(() => required),
  autocomplete: computed(() => autocomplete),
  ariaLabel: computed(() => ariaLabel),
  ariaLabelledby: computed(() => ariaLabelledby),
  ariaDescribedby: computed(() => ariaDescribedby),
  ariaErrormessage: computed(() => ariaErrormessage),
  ariaInvalid: ariaInvalidValue,
  setValue: (value: string) => {
    if (!disabled && !readOnly) valueState.set(value);
  },
  setVisible: (next: boolean) => {
    setVisible(next);
  },
  setCapsLock,
  registerInput: (element: HTMLInputElement | null) => {
    inputElement.value = element;
    if (element === null) setCapsLock(false);
  },
  focusInput: () => inputElement.value?.focus({ preventScroll: true }),
});

const slotState = computed<PasswordFieldSlotState<Strength>>(() => ({
  value: valueState.value.value,
  visible: visibleState.value.value,
  capsLock: capsLock.value,
  strength: strength.value,
  disabled,
  readOnly,
  state: dataState.value,
}));

type PasswordFieldSetupExpose = Omit<
  PasswordFieldExpose<Strength>,
  keyof PasswordFieldSlotState<Strength> | "root"
> & {
  readonly [Key in keyof PasswordFieldSlotState<Strength>]: ComputedRef<
    PasswordFieldSlotState<Strength>[Key]
  >;
} & { readonly root: typeof root };

function field<Key extends keyof PasswordFieldSlotState<Strength>>(
  key: Key,
): ComputedRef<PasswordFieldSlotState<Strength>[Key]> {
  return computed(() => slotState.value[key]);
}

const exposed = {
  value: field("value"),
  visible: field("visible"),
  capsLock: field("capsLock"),
  strength: field("strength"),
  disabled: field("disabled"),
  readOnly: field("readOnly"),
  state: field("state"),
  root,
  focus: (options?: FocusOptions) => inputElement.value?.focus(options),
  setVisible,
  toggleVisible: () => {
    setVisible(!visibleState.value.value);
    return visibleState.value.value;
  },
  setValue: (value: string) => valueState.set(value),
  reset: () => {
    valueState.reset();
    visibleState.reset();
  },
} satisfies PasswordFieldSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="root"
    part="root"
    data-vize-ui="password-field"
    :data-state="dataState"
    :data-visible="visibleState.value.value ? 'true' : 'false'"
    :data-caps-lock="capsLock ? 'true' : undefined"
    :data-disabled="disabled ? 'true' : undefined"
    :data-invalid="ariaInvalidValue === undefined ? undefined : 'true'"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
