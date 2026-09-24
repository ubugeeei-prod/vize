<script setup lang="ts" generic="T">
import { computed, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { tagsInputContext } from "./tags-input-context.ts";
import type {
  TagsInputAddSource,
  TagsInputAriaInvalid,
  TagsInputBy,
  TagsInputDirection,
  TagsInputInvalidEvent,
  TagsInputRemoveSource,
  TagsInputRootExpose,
  TagsInputSlotState,
  TagsInputState,
  TagsInputValidator,
} from "./tags-input-types.ts";
import {
  appendTag,
  areTagListsEqual,
  containsTagDelimiter,
  defaultTagText,
  evaluateTag,
  removeTagAt,
  replaceTagAt,
  resolveTagEquality,
  splitTagText,
  splitTrailingTagText,
} from "./tags-input-value.ts";
import type { TagsInputHiddenEntry } from "./tags-input-value.ts";

const {
  id = undefined,
  modelValue = undefined,
  defaultValue = undefined,
  parseTag = undefined,
  tagText = undefined,
  by = undefined,
  validate = undefined,
  delimiters = [","],
  addOnPaste = true,
  addOnBlur = false,
  allowDuplicates = false,
  max = undefined,
  editable = false,
  disabled = false,
  readonly = false,
  required = false,
  name = undefined,
  form = undefined,
  dir = "ltr",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  ariaInvalid = false,
} = defineProps<{
  /**
   * Id of the native text input, so a `<label for>` can name the field.
   * `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled tags. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: readonly T[];

  /**
   * Initial uncontrolled tags and the value restored by reset and native form reset.
   *
   * @default []
   */
  readonly defaultValue?: readonly T[];

  /**
   * Convert trimmed input text into a tag, or return `null` to reject it with
   * reason `"parse"`. Omitting it keeps the text as-is, which is only sound for
   * string tags; non-string tag types must provide a parser.
   *
   * @default undefined
   */
  readonly parseTag?: (text: string) => T | null;

  /**
   * Display and form-submission text for one tag.
   *
   * @default String(tag)
   */
  readonly tagText?: (tag: T) => string;

  /**
   * Duplicate-detection equality: an object property name or a comparator.
   *
   * @default Object.is
   */
  readonly by?: TagsInputBy<T>;

  /**
   * Custom validation. Return `false` or a message string to reject a candidate.
   *
   * @default undefined
   */
  readonly validate?: TagsInputValidator<T>;

  /**
   * Strings that commit the current text. Enter always commits.
   *
   * @default [","]
   */
  readonly delimiters?: readonly string[];

  /**
   * Split pasted text on delimiters, line breaks, and tabs into several tags.
   *
   * @default true
   */
  readonly addOnPaste?: boolean;

  /**
   * Commit pending text when the input loses focus.
   *
   * @default false
   */
  readonly addOnBlur?: boolean;

  /**
   * Accept tags equal to an existing tag under `by`.
   *
   * @default false
   */
  readonly allowDuplicates?: boolean;

  /**
   * Maximum number of tags. `undefined` means unlimited.
   *
   * @default undefined
   */
  readonly max?: number;

  /**
   * Allow Enter, F2, or double-click on a focused tag to edit it inline.
   *
   * @default false
   */
  readonly editable?: boolean;

  /**
   * Disable input, tag focus, removal, and form submission.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Keep tags focusable and submitted while preventing every change.
   *
   * @default false
   */
  readonly readonly?: boolean;

  /**
   * Require at least one tag for native constraint validation.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Native form field name. Each tag submits one entry under this name.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Id of the form owning the hidden submission inputs.
   *
   * @default undefined
   */
  readonly form?: string;

  /**
   * Reading direction used by ArrowLeft and ArrowRight between tags.
   *
   * @default "ltr"
   */
  readonly dir?: TagsInputDirection;

  /**
   * Accessible name for the text input when no label supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the text input.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the text input.
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
   * Invalid state announced on the text input.
   *
   * @default false
   */
  readonly ariaInvalid?: TagsInputAriaInvalid;
}>();

const emit = defineEmits<{
  /** Fired when the tag list requests a new controlled value. */
  "update:modelValue": [value: readonly T[]];

  /** Fired after a tag is accepted, with its index and input source. */
  add: [tag: T, index: number, source: TagsInputAddSource];

  /** Fired after a tag is removed, with its former index and input source. */
  remove: [tag: T, index: number, source: TagsInputRemoveSource];

  /** Fired after an inline edit replaces a tag. */
  edit: [tag: T, previous: T, index: number];

  /** Fired when a candidate tag is rejected by parsing, duplicates, max, or validation. */
  invalid: [event: TagsInputInvalidEvent<T>];
}>();

defineSlots<{
  /** TagsInputItem children and the TagsInputInput. Receives the current tags and state. */
  default(props: TagsInputSlotState<T>): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const inputId = useDeterministicId({ id: () => id, hint: "tags-input" });
const emptyTags: readonly T[] = Object.freeze([]);
const valueState = useControllableState<readonly T[]>({
  value: () => modelValue,
  defaultValue: () => defaultValue ?? emptyTags,
  equals: areTagListsEqual,
  onChange: (value) => emit("update:modelValue", value),
});
const tags = computed(() => valueState.value.value);
const inputValue = shallowRef("");
const editingIndex = shallowRef<number | null>(null);
const activeIndex = shallowRef<number | null>(null);
const disabledState = computed(() => disabled);
const readonlyState = computed(() => readonly);
const requiredState = computed(() => required);
const editableState = computed(() => editable && !disabled && !readonly);
const directionState = computed(() => dir);
const addOnBlurState = computed(() => addOnBlur);
const ariaInvalidValue = computed(() => (ariaInvalid === false ? undefined : ariaInvalid));
const invalid = computed(() => ariaInvalidValue.value !== undefined);
const full = computed(() => max !== undefined && tags.value.length >= max);
const locked = computed(() => disabled || readonly);
const state = computed<TagsInputState>(() => {
  if (disabled) return "disabled";
  if (readonly) return "readonly";
  return tags.value.length === 0 ? "empty" : "filled";
});
const count = computed<number>(() => tags.value.length);
const hiddenEntries = computed<readonly TagsInputHiddenEntry[]>(() =>
  tags.value.map((tag, index) => {
    const value = textOf(tag);
    return { key: `${index}:${value}`, value };
  }),
);
const slotState = computed<TagsInputSlotState<T>>(() => ({
  count: tags.value.length,
  disabled,
  full: full.value,
  inputValue: inputValue.value,
  invalid: invalid.value,
  readonly,
  state: state.value,
  tags: tags.value,
}));

function ownedElement(elementId: string): HTMLElement | null {
  const found = element.value?.ownerDocument.getElementById(elementId) ?? null;
  return found !== null && element.value?.contains(found) === true ? found : null;
}

function itemId(index: number): string {
  return deriveDeterministicId(inputId.value, `tag-${index}`);
}

function equals(left: T, right: T): boolean {
  return resolveTagEquality(by)(left, right);
}

function textOf(tag: T): string {
  return tagText === undefined ? defaultTagText(tag) : tagText(tag);
}

function parse(text: string): T | null {
  if (parseTag !== undefined) return parseTag(text);
  // Documented contract: without `parseTag`, tags are the typed strings.
  return text as unknown as T;
}

function reject(
  text: string,
  tag: T | null,
  reason: TagsInputInvalidEvent<T>["reason"],
  message: string | null,
): false {
  emit("invalid", Object.freeze({ message, reason, tag, text }));
  return false;
}

function tryAddTag(tag: T, text: string, source: TagsInputAddSource): boolean {
  if (locked.value) return false;
  const rejection = evaluateTag(tag, tags.value, {
    allowDuplicates,
    equals,
    max,
    validate,
  });
  if (rejection !== null) return reject(text, tag, rejection.reason, rejection.message);
  const next = appendTag(tags.value, tag);
  valueState.set(next);
  emit("add", tag, next.length - 1, source);
  return true;
}

function tryAddText(text: string, source: TagsInputAddSource): boolean {
  if (locked.value) return false;
  const trimmed = text.trim();
  if (trimmed.length === 0) return false;
  const tag = parse(trimmed);
  if (tag === null) return reject(trimmed, null, "parse", null);
  return tryAddTag(tag, trimmed, source);
}

function commitSegments(segments: readonly string[], source: TagsInputAddSource): string[] {
  return segments.filter((segment) => !tryAddText(segment, source));
}

function restoreRejected(rejected: readonly string[], rest: string): string {
  // Rejected candidates stay editable without re-inserting a delimiter, so the
  // next keystroke cannot silently re-submit them.
  const joiner = delimiters.includes(" ") ? "" : " ";
  return [...rejected, rest].filter((part) => part.length > 0).join(joiner);
}

function commitInput(source: "blur" | "delimiter" | "enter"): boolean {
  if (locked.value) return false;
  const segments = splitTagText(inputValue.value, delimiters);
  if (segments.length === 0) {
    inputValue.value = "";
    return false;
  }
  const rejected = commitSegments(segments, source);
  inputValue.value = restoreRejected(rejected, "");
  return rejected.length < segments.length;
}

function handleInputText(text: string): void {
  if (locked.value || !containsTagDelimiter(text, delimiters)) {
    inputValue.value = text;
    return;
  }
  const { complete, rest } = splitTrailingTagText(text, delimiters);
  const rejected = commitSegments(splitTagText(complete, delimiters), "delimiter");
  inputValue.value = restoreRejected(rejected, rest);
}

function handlePaste(pasted: string, selectionStart: number, selectionEnd: number): boolean {
  if (locked.value || !addOnPaste || !containsTagDelimiter(pasted, delimiters, true)) {
    return false;
  }
  const combined =
    inputValue.value.slice(0, selectionStart) + pasted + inputValue.value.slice(selectionEnd);
  const rejected = commitSegments(splitTagText(combined, delimiters, true), "paste");
  inputValue.value = restoreRejected(rejected, "");
  return true;
}

function isDelimiterKey(key: string): boolean {
  return key.length > 0 && delimiters.includes(key);
}

function removeAt(index: number, source: TagsInputRemoveSource): boolean {
  if (locked.value) return false;
  if (!Number.isInteger(index) || index < 0 || index >= tags.value.length) return false;
  // Bounds are checked above, so the indexed read is a present tag.
  const tag = tags.value[index] as T;
  valueState.set(removeTagAt(tags.value, index));
  if (editingIndex.value === index) editingIndex.value = null;
  emit("remove", tag, index, source);
  return true;
}

function add(text: string): boolean {
  return tryAddText(text, "api");
}

function addTag(tag: T): boolean {
  return tryAddTag(tag, textOf(tag), "api");
}

function remove(index: number): boolean {
  return removeAt(index, "api");
}

function clear(): boolean {
  if (locked.value) return false;
  return valueState.set(emptyTags);
}

function setInputValue(text: string): void {
  inputValue.value = text;
}

function reset(): boolean {
  inputValue.value = "";
  editingIndex.value = null;
  return valueState.reset();
}

function focus(options?: FocusOptions): void {
  ownedElement(inputId.value)?.focus(options);
}

function focusItem(index: number): boolean {
  const target = disabled ? null : ownedElement(itemId(index));
  if (target === null || target.getAttribute("tabindex") === null) return false;
  target.focus();
  return true;
}

function startEdit(index: number): boolean {
  if (!editableState.value || index < 0 || index >= tags.value.length) return false;
  editingIndex.value = index;
  return true;
}

function commitEdit(index: number, text: string, keepEditingOnInvalid: boolean): boolean {
  if (editingIndex.value !== index || locked.value) return false;
  const previous = tags.value[index] as T;
  const trimmed = text.trim();
  const tag = trimmed.length === 0 ? null : parse(trimmed);
  if (tag === null) {
    if (trimmed.length > 0) reject(trimmed, null, "parse", null);
    if (!keepEditingOnInvalid || trimmed.length === 0) cancelEdit(index);
    return false;
  }
  const rejection = evaluateTag(tag, tags.value, {
    allowDuplicates,
    equals,
    max,
    replaceIndex: index,
    validate,
  });
  if (rejection !== null) {
    reject(trimmed, tag, rejection.reason, rejection.message);
    if (!keepEditingOnInvalid) cancelEdit(index);
    return false;
  }
  editingIndex.value = null;
  if (!Object.is(previous, tag)) {
    valueState.set(replaceTagAt(tags.value, index, tag));
    emit("edit", tag, previous, index);
  }
  return true;
}

function cancelEdit(index: number): void {
  if (editingIndex.value === index) editingIndex.value = null;
}

function onRootPointerdown(event: PointerEvent): void {
  if (disabled || event.target !== event.currentTarget) return;
  event.preventDefault();
  focus();
}

watch(
  element,
  (root, _previous, onCleanup) => {
    const owner = root?.closest("form");
    if (owner === undefined || owner === null) return;
    const onReset = () => {
      inputValue.value = "";
      editingIndex.value = null;
      if (!valueState.controlled.value) valueState.reset();
    };
    owner.addEventListener("reset", onReset);
    onCleanup(() => owner.removeEventListener("reset", onReset));
  },
  { flush: "post", immediate: true },
);

watch(tags, (next) => {
  if (editingIndex.value !== null && editingIndex.value >= next.length) editingIndex.value = null;
  if (activeIndex.value !== null && activeIndex.value >= next.length) activeIndex.value = null;
});

tagsInputContext.provide({
  activeIndex,
  addOnBlur: addOnBlurState,
  ariaDescribedby: computed(() => ariaDescribedby),
  ariaErrormessage: computed(() => ariaErrormessage),
  ariaInvalid: ariaInvalidValue,
  ariaLabel: computed(() => ariaLabel),
  ariaLabelledby: computed(() => ariaLabelledby),
  cancelEdit,
  commitEdit,
  commitInput,
  direction: directionState,
  disabled: disabledState,
  editable: editableState,
  editingIndex,
  focusInput: () => focus(),
  focusItem,
  handleInputText,
  handlePaste,
  inputId,
  inputValue,
  isDelimiterKey,
  itemId,
  readonly: readonlyState,
  removeAt,
  required: requiredState,
  startEdit,
  tagText: (tag) => textOf(tag as T),
  tags,
});

type TagsInputRootSetupExpose = Omit<
  TagsInputRootExpose<T>,
  "element" | "id" | "inputValue" | "state" | "tags"
> & {
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly inputValue: typeof inputValue;
  readonly state: ComputedRef<TagsInputState>;
  readonly tags: ComputedRef<readonly T[]>;
};

const exposed = {
  add,
  addTag,
  clear,
  element,
  focus,
  id: inputId,
  inputValue,
  remove,
  reset,
  setInputValue,
  state,
  tags,
} satisfies TagsInputRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    data-vize-ui="tags-input"
    part="root"
    :dir
    :data-state="state"
    :data-disabled="disabled ? 'true' : undefined"
    :data-readonly="readonly ? 'true' : undefined"
    :data-invalid="invalid ? 'true' : undefined"
    :data-full="full ? 'true' : undefined"
    :data-count="count"
    @pointerdown="onRootPointerdown"
  >
    <slot v-bind="slotState" />
    <template v-if="name !== undefined">
      <input
        v-for="entry in hiddenEntries as readonly TagsInputHiddenEntry[]"
        :key="entry.key"
        type="hidden"
        :name
        :form
        :value="entry.value"
        :disabled
        data-vize-ui="tags-input-hidden-input"
      />
    </template>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
