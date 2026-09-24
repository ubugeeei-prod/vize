<script setup lang="ts" generic="Length extends number">
import { computed, nextTick, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { pinInputContext } from "./pin-input-context.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import {
  isPinCharacters,
  normalizePinLength,
  removePinCharacter,
  sanitizePinInput,
  splitPinCharacters,
  writePinCharacters,
} from "./pin-input-state.ts";
import type {
  PinInputAriaInvalid,
  PinInputCharacters,
  PinInputExpose,
  PinInputSlotState,
  PinInputState,
  PinInputType,
} from "./pin-input-types.ts";

const {
  length,
  modelValue = undefined,
  defaultValue = "",
  type = "numeric",
  pattern = undefined,
  mask = false,
  otp = true,
  id = undefined,
  name = undefined,
  form = undefined,
  placeholder = undefined,
  disabled = false,
  required = false,
  getFieldLabel = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  ariaInvalid = false,
} = defineProps<{
  /**
   * Number of fields. A literal length types the `complete` characters tuple.
   *
   * @default required
   */
  readonly length: Length;

  /**
   * Controlled code. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: string;

  /**
   * Initial uncontrolled code, also restored by form reset.
   *
   * @default ""
   */
  readonly defaultValue?: string;

  /**
   * Accepted characters: decimal digits, or letters and digits.
   *
   * @default "numeric"
   */
  readonly type?: PinInputType;

  /**
   * Single-character pattern that overrides `type`, for example `/[0-9A-F]/`.
   *
   * @default undefined
   */
  readonly pattern?: RegExp;

  /**
   * Render fields as password inputs so entered characters are masked.
   *
   * @default false
   */
  readonly mask?: boolean;

  /**
   * One-time-code mode: the first field advertises `autocomplete="one-time-code"`
   * so browsers can autofill SMS codes.
   *
   * @default true
   */
  readonly otp?: boolean;

  /**
   * Base id; fields use `<id>-<index>`. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Native form field name. A hidden input submits the joined code.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Id of a form owner outside the component tree.
   *
   * @default undefined
   */
  readonly form?: string;

  /**
   * Placeholder shown in each empty field, for example `"○"`.
   *
   * @default undefined
   */
  readonly placeholder?: string;

  /**
   * Disable every field and native form submission.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Require every field for native constraint validation.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Accessible name of each field.
   *
   * @default (index, length) => `Character ${index + 1} of ${length}`
   */
  readonly getFieldLabel?: (index: number, length: number) => string;

  /**
   * Accessible name of the group when no label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the group.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe every field.
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
   * Invalid state announced on every field.
   *
   * @default false
   */
  readonly ariaInvalid?: PinInputAriaInvalid;
}>();

const emit = defineEmits<{
  /** Fired when the joined code requests a new controlled value. */
  "update:modelValue": [value: string];

  /** Fired when the last field is filled, with the code and its characters as a typed tuple. */
  complete: [value: string, characters: PinInputCharacters<Length>];
}>();

defineSlots<{
  /** Renders one PinInputField per entry in `indexes`, plus any separators. */
  default(props: PinInputSlotState): unknown;
}>();

const root = useTemplateRef<HTMLDivElement>("root");
const baseId = useDeterministicId({ id: () => id, hint: "pin" });
const fieldCount = computed(() => normalizePinLength(length));
const fields = new Map<number, HTMLInputElement>();

function sanitize(text: string): string {
  return sanitizePinInput(text, type, pattern).slice(0, fieldCount.value).join("");
}

const state = useControllableState({
  value: () => (modelValue === undefined ? undefined : sanitize(modelValue)),
  defaultValue: () => sanitize(defaultValue),
  onChange: (next, previous) => {
    emit("update:modelValue", next);
    const nextCharacters = splitPinCharacters(next);
    const wasComplete = splitPinCharacters(previous).length === fieldCount.value;
    if (!wasComplete && isPinCharacters(nextCharacters, length)) {
      emit("complete", next, nextCharacters);
    }
  },
});
const value = state.value;
const characters = computed(() => splitPinCharacters(value.value));
const complete = computed(() => characters.value.length === fieldCount.value);
const indexes = computed(() => Array.from({ length: fieldCount.value }, (_, index) => index));
const ariaInvalidValue = computed(() => {
  if (ariaInvalid === false) return undefined;
  return ariaInvalid === true ? "true" : ariaInvalid;
});
const dataState = computed<PinInputState>(() => {
  if (disabled) return "disabled";
  if (complete.value) return "complete";
  return characters.value.length === 0 ? "empty" : "incomplete";
});

function commit(next: string): boolean {
  return state.set(next);
}

function focusField(index: number): void {
  const clamped = Math.min(Math.max(index, 0), fieldCount.value - 1);
  const element = fields.get(clamped);
  element?.focus();
  element?.select();
}

/** Write typed or pasted text at a field; returns the index that should receive focus. */
function write(index: number, text: string): number {
  if (disabled) return index;
  const accepted = sanitizePinInput(text, type, pattern);
  if (accepted.length === 0) return index;
  const start = Math.min(index, characters.value.length);
  commit(writePinCharacters(value.value, start, accepted, fieldCount.value));
  return Math.min(start + accepted.length, fieldCount.value - 1);
}

/** Delete a character; returns the index that should receive focus. */
function remove(index: number, direction: "backward" | "forward"): number {
  if (disabled) return index;
  const hasCharacter = index < characters.value.length;
  if (direction === "forward" || hasCharacter) {
    commit(removePinCharacter(value.value, index));
    return index;
  }
  const previous = Math.min(index, characters.value.length) - 1;
  if (previous < 0) return 0;
  commit(removePinCharacter(value.value, previous));
  return previous;
}

watch(
  root,
  (element, _previous, onCleanup) => {
    const owner =
      form === undefined ? element?.closest("form") : element?.ownerDocument.getElementById(form);
    if (!(owner instanceof HTMLFormElement)) return;
    const onReset = () => {
      if (!state.controlled.value) state.reset();
      // Native reset clears field values first; re-apply the restored characters after it.
      void nextTick(() => {
        for (const [index, input] of fields) input.value = characters.value[index] ?? "";
      });
    };
    owner.addEventListener("reset", onReset);
    onCleanup(() => owner.removeEventListener("reset", onReset));
  },
  { flush: "post", immediate: true },
);

function defaultFieldLabel(index: number, count: number): string {
  return `Character ${index + 1} of ${count}`;
}

pinInputContext.provide({
  baseId,
  characters,
  length: fieldCount,
  type: computed(() => type),
  mask: computed(() => mask),
  otp: computed(() => otp),
  disabled: computed(() => disabled),
  required: computed(() => required),
  complete,
  placeholder: computed(() => placeholder),
  ariaDescribedby: computed(() => ariaDescribedby),
  ariaInvalid: ariaInvalidValue,
  ariaErrormessage: computed(() => ariaErrormessage),
  fieldLabel: (index: number) => (getFieldLabel ?? defaultFieldLabel)(index, fieldCount.value),
  registerField: (index: number, element: HTMLInputElement | null) => {
    if (element === null) fields.delete(index);
    else fields.set(index, element);
  },
  focusField,
  write,
  remove,
});

const slotState = computed<PinInputSlotState>(() => ({
  value: value.value,
  length: fieldCount.value,
  indexes: indexes.value,
  complete: complete.value,
  disabled,
  state: dataState.value,
}));

type PinInputSetupExpose = Omit<PinInputExpose, keyof PinInputSlotState | "root"> & {
  readonly [Key in keyof PinInputSlotState]: ComputedRef<PinInputSlotState[Key]>;
} & { readonly root: typeof root };

function field<Key extends keyof PinInputSlotState>(key: Key): ComputedRef<PinInputSlotState[Key]> {
  return computed(() => slotState.value[key]);
}

const exposed = {
  value: field("value"),
  length: field("length"),
  indexes: field("indexes"),
  complete: field("complete"),
  disabled: field("disabled"),
  state: field("state"),
  root,
  focus: (index?: number) =>
    focusField(index ?? Math.min(characters.value.length, fieldCount.value - 1)),
  setValue: (next: string) => commit(sanitize(next)),
  clear: () => commit(""),
} satisfies PinInputSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="root"
    role="group"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    part="root"
    data-vize-ui="pin-input"
    :data-state="dataState"
    :data-complete="complete ? 'true' : 'false'"
    :data-disabled="disabled ? 'true' : undefined"
    :data-invalid="ariaInvalidValue === undefined ? undefined : 'true'"
  >
    <input
      v-if="name !== undefined"
      type="hidden"
      :name
      :form
      :value
      :disabled
      data-vize-ui="pin-input-value"
    />
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
