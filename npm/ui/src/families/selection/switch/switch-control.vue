<script setup lang="ts">
import { computed, useTemplateRef, watch } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import type {
  SwitchAriaInvalid,
  SwitchExpose,
  SwitchSlotState,
  SwitchState,
} from "./switch-types.ts";

const {
  id = undefined,
  name = undefined,
  value = "on",
  modelValue = undefined,
  defaultChecked = false,
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
   * Consumer-owned control id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Native form field name submitted while the switch is checked.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Native form value submitted while the switch is checked.
   *
   * @default "on"
   */
  readonly value?: string;

  /**
   * Controlled checked value. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: boolean;

  /**
   * Initial checked state for uncontrolled use and the state restored by reset.
   *
   * @default false
   */
  readonly defaultChecked?: boolean;

  /**
   * Disable activation, focus, and native form submission.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Keep the switch focusable while preventing user state changes.
   *
   * @default false
   */
  readonly readOnly?: boolean;

  /**
   * Mark the switch as required for assistive technology.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the switch.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the switch.
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
  readonly ariaInvalid?: SwitchAriaInvalid;
}>();

defineSlots<{
  /** Renders the switch contents with current checked and availability state. */
  default(props: SwitchSlotState): unknown;
}>();

const emit = defineEmits<{
  /** Fired when the checked state requests a new controlled boolean value. */
  "update:modelValue": [value: boolean];

  /** Fired after user activation requests the next checked value. */
  change: [value: boolean, nativeEvent: MouseEvent];
}>();

const element = useTemplateRef<HTMLButtonElement>("element");
const controlId = useDeterministicId({ id: () => id, hint: "switch" });
const state = useControllableState({
  value: () => modelValue,
  defaultValue: () => defaultChecked,
  onChange: (checked) => emit("update:modelValue", checked),
});
const checked = state.value;
const ariaChecked = computed(() => (checked.value ? "true" : "false"));
const ariaInvalidValue = computed(() => {
  if (ariaInvalid === false) return undefined;
  return ariaInvalid === true ? "true" : ariaInvalid;
});
const invalid = computed(() => ariaInvalidValue.value !== undefined);
const dataState = computed<SwitchState>(() => {
  if (disabled) return "disabled";
  if (readOnly) return "readonly";
  return checked.value ? "checked" : "unchecked";
});
const submitsValue = computed(
  () => name !== undefined && name.length > 0 && checked.value && !disabled,
);

watch(
  element,
  (button, _previous, onCleanup) => {
    const form = button?.form;
    if (form === undefined || form === null) return;
    const onReset = () => {
      if (!state.controlled.value) state.reset();
    };
    form.addEventListener("reset", onReset);
    onCleanup(() => form.removeEventListener("reset", onReset));
  },
  { flush: "post", immediate: true },
);

function onClick(event: MouseEvent): void {
  if (disabled) {
    event.preventDefault();
    event.stopImmediatePropagation();
    return;
  }
  if (readOnly) {
    event.preventDefault();
    return;
  }

  const next = !checked.value;
  state.set(next);
  emit("change", next, event);
}

/** Move focus to the native switch button. */
function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

function toggle(): boolean {
  return state.set((current) => !current);
}

type SwitchSetupExpose = Omit<SwitchExpose, "checked"> & {
  readonly checked: typeof checked;
  readonly element: typeof element;
};

const exposed = {
  checked,
  element,
  focus,
  reset: state.reset,
  setChecked: state.set,
  toggle,
} satisfies SwitchSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    :id="controlId"
    ref="element"
    type="button"
    role="switch"
    :disabled
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-errormessage="ariaInvalidValue === undefined ? undefined : ariaErrormessage"
    :aria-invalid="ariaInvalidValue"
    :aria-checked="ariaChecked"
    :aria-disabled="disabled ? 'true' : undefined"
    :aria-readonly="readOnly ? 'true' : undefined"
    :aria-required="required ? 'true' : undefined"
    data-vize-ui="switch"
    :data-state="dataState"
    :data-checked="ariaChecked"
    @click="onClick"
  >
    <slot :checked :disabled :invalid :read-only="readOnly" :required />
    <input v-if="submitsValue" type="hidden" :name :value />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
