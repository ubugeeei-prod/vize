<script setup lang="ts">
import { computed, nextTick, shallowRef, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { editableContext } from "./editable-context.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import type {
  EditableActivationMode,
  EditableAriaInvalid,
  EditableExpose,
  EditableSlotState,
  EditableState,
  EditableSubmitMode,
} from "./editable-types.ts";

const {
  id = undefined,
  name = undefined,
  modelValue = undefined,
  defaultValue = "",
  editing = undefined,
  defaultEditing = false,
  activationMode = "focus",
  submitMode = "both",
  selectOnFocus = true,
  placeholder = undefined,
  maxLength = undefined,
  disabled = false,
  readOnly = false,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  ariaInvalid = false,
} = defineProps<{
  /**
   * Id of the edit input. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Native form field name; a hidden input submits the committed value.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Controlled committed value. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: string;

  /**
   * Initial uncontrolled value.
   *
   * @default ""
   */
  readonly defaultValue?: string;

  /**
   * Controlled edit mode (`v-model:editing`). `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly editing?: boolean;

  /**
   * Initial uncontrolled edit mode.
   *
   * @default false
   */
  readonly defaultEditing?: boolean;

  /**
   * How the preview enters edit mode. Enter and F2 on a focused preview always do.
   *
   * @default "focus"
   */
  readonly activationMode?: EditableActivationMode;

  /**
   * Which interactions commit the draft: Enter, blur, both, or only an explicit submit.
   *
   * @default "both"
   */
  readonly submitMode?: EditableSubmitMode;

  /**
   * Select the input text when edit mode starts.
   *
   * @default true
   */
  readonly selectOnFocus?: boolean;

  /**
   * Text shown by the preview and input while empty.
   *
   * @default undefined
   */
  readonly placeholder?: string;

  /**
   * Native maximum length of the input.
   *
   * @default undefined
   */
  readonly maxLength?: number;

  /**
   * Prevent editing and remove the preview from the tab order.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Show the value without allowing edit mode.
   *
   * @default false
   */
  readonly readOnly?: boolean;

  /**
   * Accessible name of the input when no label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the input and preview.
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
   * Invalid state announced on the input.
   *
   * @default false
   */
  readonly ariaInvalid?: EditableAriaInvalid;
}>();

const emit = defineEmits<{
  /** Fired when a submit requests a new committed value. */
  "update:modelValue": [value: string];

  /** Fired when edit mode requests to open or close (`v-model:editing`). */
  "update:editing": [editing: boolean];

  /** Fired when edit mode starts. */
  edit: [];

  /** Fired after a submit with the committed value and the previous value. */
  submit: [value: string, previous: string];

  /** Fired after a cancel with the restored value and the discarded draft. */
  cancel: [value: string, discarded: string];
}>();

defineSlots<{
  /** Renders the preview, input, and triggers with the editing state. */
  default(props: EditableSlotState): unknown;
}>();

const root = useTemplateRef<HTMLDivElement>("root");
const inputElement = shallowRef<HTMLInputElement | null>(null);
const previewElement = shallowRef<HTMLElement | null>(null);
const inputId = useDeterministicId({ id: () => id, hint: "editable" });
const valueState = useControllableState({
  value: () => modelValue,
  defaultValue: () => defaultValue,
  onChange: (value) => emit("update:modelValue", value),
});
const editingState = useControllableState({
  value: () => editing,
  defaultValue: () => defaultEditing,
  onChange: (value) => emit("update:editing", value),
});
const draftText = shallowRef<string | null>(null);
const draft = computed(() => draftText.value ?? valueState.value.value);
const interactive = computed(() => !disabled && !readOnly);
const isEditing = computed(() => interactive.value && editingState.value.value);
const ariaInvalidValue = computed(() => {
  if (ariaInvalid === false) return undefined;
  return ariaInvalid === true ? "true" : ariaInvalid;
});
const dataState = computed<EditableState>(() => {
  if (disabled) return "disabled";
  if (readOnly) return "readonly";
  return isEditing.value ? "editing" : "preview";
});
let returningFocus = false;

function focusInput(): void {
  void nextTick(() => {
    inputElement.value?.focus();
    if (selectOnFocus) inputElement.value?.select();
  });
}

/** Move focus to the preview only when it would otherwise be lost with the hidden input. */
function returnFocus(): void {
  if (inputElement.value?.ownerDocument.activeElement !== inputElement.value) return;
  void nextTick(() => {
    // Suppress focus activation so returning focus does not re-open edit mode.
    returningFocus = true;
    previewElement.value?.focus();
    returningFocus = false;
  });
}

function edit(): boolean {
  if (!interactive.value || isEditing.value) return false;
  draftText.value = valueState.value.value;
  editingState.set(true);
  emit("edit");
  focusInput();
  return true;
}

function submit(): boolean {
  if (!isEditing.value) return false;
  const previous = valueState.value.value;
  const changed = valueState.set(draft.value);
  emit("submit", draft.value, previous);
  returnFocus();
  draftText.value = null;
  editingState.set(false);
  return changed;
}

function cancel(): void {
  if (!isEditing.value) return;
  emit("cancel", valueState.value.value, draft.value);
  returnFocus();
  draftText.value = null;
  editingState.set(false);
}

editableContext.provide({
  inputId,
  value: valueState.value,
  draft,
  editing: isEditing,
  interactive,
  disabled: computed(() => disabled),
  readOnly: computed(() => readOnly),
  state: dataState,
  placeholder: computed(() => placeholder),
  maxLength: computed(() => maxLength),
  activationMode: computed(() => activationMode),
  submitOnBlur: computed(() => submitMode === "blur" || submitMode === "both"),
  submitOnEnter: computed(() => submitMode === "enter" || submitMode === "both"),
  ariaLabel: computed(() => ariaLabel),
  ariaLabelledby: computed(() => ariaLabelledby),
  ariaDescribedby: computed(() => ariaDescribedby),
  ariaErrormessage: computed(() => ariaErrormessage),
  ariaInvalid: ariaInvalidValue,
  setDraft: (text: string) => {
    draftText.value = text;
  },
  edit,
  submit,
  cancel,
  registerInput: (element: HTMLInputElement | null) => {
    inputElement.value = element;
  },
  registerPreview: (element: HTMLElement | null) => {
    previewElement.value = element;
  },
  consumeFocusReturn: () => returningFocus,
});

const slotState = computed<EditableSlotState>(() => ({
  value: valueState.value.value,
  draft: draft.value,
  editing: isEditing.value,
  empty: valueState.value.value.length === 0,
  disabled,
  readOnly,
  state: dataState.value,
}));

type EditableSetupExpose = Omit<EditableExpose, keyof EditableSlotState | "root"> & {
  readonly [Key in keyof EditableSlotState]: ComputedRef<EditableSlotState[Key]>;
} & { readonly root: typeof root };

function field<Key extends keyof EditableSlotState>(key: Key): ComputedRef<EditableSlotState[Key]> {
  return computed(() => slotState.value[key]);
}

const exposed = {
  value: field("value"),
  draft: field("draft"),
  editing: field("editing"),
  empty: field("empty"),
  disabled: field("disabled"),
  readOnly: field("readOnly"),
  state: field("state"),
  root,
  edit,
  submit,
  cancel,
  setValue: (next: string) => valueState.set(next),
} satisfies EditableSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="root"
    part="root"
    data-vize-ui="editable"
    :data-state="dataState"
    :data-empty="valueState.value.value.length === 0 ? 'true' : undefined"
    :data-disabled="disabled ? 'true' : undefined"
    :data-invalid="ariaInvalidValue === undefined ? undefined : 'true'"
  >
    <input
      v-if="name !== undefined"
      type="hidden"
      :name
      :value="valueState.value.value"
      :disabled
      data-vize-ui="editable-value"
    />
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
