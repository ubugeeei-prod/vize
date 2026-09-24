<script setup lang="ts" generic="Value">
import { computed, nextTick, toRaw, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { checkboxGroupContext } from "./checkbox-group-context.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import type {
  CheckboxGroupAriaInvalid,
  CheckboxGroupExpose,
  CheckboxGroupItemState,
  CheckboxGroupKey,
  CheckboxGroupOrientation,
  CheckboxGroupSlotState,
  CheckboxGroupState,
} from "./checkbox-group-types.ts";

const {
  options,
  modelValue = undefined,
  defaultValue = [],
  by = undefined,
  getFormValue = undefined,
  isOptionDisabled = undefined,
  name = undefined,
  form = undefined,
  disabled = false,
  required = false,
  orientation = "vertical",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  ariaInvalid = false,
} = defineProps<{
  /**
   * Every selectable value, in display order. Infers the group's value type.
   *
   * @default required
   */
  readonly options: readonly Value[];

  /**
   * Controlled selected values. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: readonly Value[];

  /**
   * Initial uncontrolled selection, also restored by form reset.
   *
   * @default []
   */
  readonly defaultValue?: readonly Value[];

  /**
   * Identity key for comparing selected values with options, for example
   * `(user) => user.id` when the model holds fresh object copies.
   *
   * @default Object.is identity
   */
  readonly by?: (value: Value) => CheckboxGroupKey;

  /**
   * Native form value submitted for an option.
   *
   * @default String(value) for primitives, else the option index
   */
  readonly getFormValue?: (value: Value, index: number) => string;

  /**
   * Disable individual options. Disabled options keep their selection when
   * select-all toggles.
   *
   * @default undefined
   */
  readonly isOptionDisabled?: (value: Value, index: number) => boolean;

  /**
   * Native form field name shared by every item checkbox.
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
   * Disable every checkbox in the group.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Require at least one selection for native constraint validation.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Layout direction published through `data-orientation`.
   *
   * @default "vertical"
   */
  readonly orientation?: CheckboxGroupOrientation;

  /**
   * Accessible name when no label or `aria-labelledby` supplies one.
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
   * Space-separated ids that describe the group and every checkbox.
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
   * Invalid state announced on every checkbox.
   *
   * @default false
   */
  readonly ariaInvalid?: CheckboxGroupAriaInvalid;
}>();

const emit = defineEmits<{
  /** Fired when the selection requests new values (in option order). */
  "update:modelValue": [value: readonly Value[]];

  /** Fired after a user or API change with the next values, the toggled value (or `null` for select-all), and its new state. */
  change: [value: readonly Value[], toggled: Value | null, selected: boolean];
}>();

defineSlots<{
  /** Renders items and the optional select-all parent with the selection summary. */
  default(props: CheckboxGroupSlotState<Value>): unknown;
}>();

const root = useTemplateRef<HTMLDivElement>("root");

/** Identity through reactive proxies, so `reactive()` option lists still match raw values. */
function sameIdentity(left: unknown, right: unknown): boolean {
  return Object.is(toRaw(left), toRaw(right));
}

function sameValue(left: Value, right: Value): boolean {
  return by === undefined ? sameIdentity(left, right) : Object.is(by(left), by(right));
}

function optionIndex(value: Value): number {
  return options.findIndex((option) => sameValue(option, value));
}

/** Box one option so `undefined` options stay distinguishable from "missing". */
function optionAt(index: number): { readonly value: Value } | undefined {
  for (const [position, option] of options.entries()) {
    if (position === index) return { value: option };
  }
  return undefined;
}

/** Keep only known options, deduplicated, in option order. */
function normalize(values: readonly Value[]): readonly Value[] {
  const indexes = new Set(values.map((value) => optionIndex(value)));
  return options.filter((_, index) => indexes.has(index));
}

function sameSelection(left: readonly Value[], right: readonly Value[]): boolean {
  const rightIndexes = right.map((value) => optionIndex(value));
  return (
    left.length === right.length &&
    left.every((value, index) => optionIndex(value) === rightIndexes[index])
  );
}

const state = useControllableState<readonly Value[]>({
  value: () => (modelValue === undefined ? undefined : normalize(modelValue)),
  defaultValue: () => normalize(defaultValue),
  equals: sameSelection,
  onChange: (value) => emit("update:modelValue", value),
});
const values = state.value;
const selectedIndexes = computed(() => new Set(values.value.map((value) => optionIndex(value))));

const disabledOptionIndexes = computed(
  () =>
    new Set(
      options.flatMap((option, index) =>
        isOptionDisabled?.(option, index) === true ? [index] : [],
      ),
    ),
);

function isIndexDisabled(index: number): boolean {
  if (disabled || index < 0 || index >= options.length) return true;
  return disabledOptionIndexes.value.has(index);
}

const enabledIndexes = computed(() =>
  options.map((_, index) => index).filter((index) => !isIndexDisabled(index)),
);
const allSelected = computed(
  () =>
    enabledIndexes.value.length > 0 &&
    enabledIndexes.value.every((index) => selectedIndexes.value.has(index)),
);
const someSelected = computed(
  () =>
    !allSelected.value && enabledIndexes.value.some((index) => selectedIndexes.value.has(index)),
);
const dataState = computed<CheckboxGroupState>(() => {
  if (disabled) return "disabled";
  if (allSelected.value) return "all";
  return someSelected.value || values.value.length > 0 ? "some" : "none";
});
const selectAllState = computed<CheckboxGroupItemState>(() => {
  if (allSelected.value) return "checked";
  return someSelected.value ? "indeterminate" : "unchecked";
});
const ariaInvalidValue = computed(() => {
  if (ariaInvalid === false) return undefined;
  return ariaInvalid === true ? "true" : ariaInvalid;
});

function commitIndexes(
  indexes: ReadonlySet<number>,
  toggled: Value | null,
  selected: boolean,
): boolean {
  const next = options.filter((_, index) => indexes.has(index));
  const changed = state.set(next);
  if (changed) emit("change", next, toggled, selected);
  return changed;
}

function toggleIndex(index: number, selected: boolean): boolean {
  if (isIndexDisabled(index)) return false;
  const next = new Set(selectedIndexes.value);
  if (selected) next.add(index);
  else next.delete(index);
  return commitIndexes(next, optionAt(index)?.value ?? null, selected);
}

function toggleAll(selected: boolean): boolean {
  if (disabled) return false;
  const next = new Set(selectedIndexes.value);
  for (const index of enabledIndexes.value) {
    if (selected) next.add(index);
    else next.delete(index);
  }
  return commitIndexes(next, null, selected);
}

function formValueOf(index: number): string {
  const entry = optionAt(index);
  if (entry === undefined) return "";
  const option = entry.value;
  if (getFormValue !== undefined) return getFormValue(option, index);
  const kind = typeof option;
  return kind === "string" || kind === "number" || kind === "bigint" || kind === "boolean"
    ? String(option)
    : String(index);
}

function indexOf(candidate: unknown): number {
  return options.findIndex((option) => sameIdentity(option, candidate));
}

watch(
  root,
  (element, _previous, onCleanup) => {
    const owner =
      form === undefined ? element?.closest("form") : element?.ownerDocument.getElementById(form);
    if (!(owner instanceof HTMLFormElement)) return;
    const onReset = () => {
      if (!state.controlled.value) state.reset();
      // Native reset restores `checked` attributes first; re-apply our state after it.
      void nextTick(() => {
        for (const input of element?.querySelectorAll<HTMLInputElement>("input[data-index]") ??
          []) {
          input.checked = selectedIndexes.value.has(Number(input.dataset.index));
        }
      });
    };
    owner.addEventListener("reset", onReset);
    onCleanup(() => owner.removeEventListener("reset", onReset));
  },
  { flush: "post", immediate: true },
);

checkboxGroupContext.provide({
  name: computed(() => name),
  form: computed(() => form),
  required: computed(() => required),
  groupDisabled: computed(() => disabled),
  hasSelection: computed(() => values.value.length > 0),
  selectAllState,
  selectAllDisabled: computed(() => disabled || enabledIndexes.value.length === 0),
  indexOf,
  isIndexSelected: (index: number) => selectedIndexes.value.has(index),
  isIndexDisabled,
  formValueOf,
  toggleIndex,
  toggleAll,
  ariaDescribedby: computed(() => ariaDescribedby),
  ariaInvalid: ariaInvalidValue,
  ariaErrormessage: computed(() => ariaErrormessage),
});

const slotState = computed<CheckboxGroupSlotState<Value>>(() => ({
  values: values.value,
  options,
  allSelected: allSelected.value,
  someSelected: someSelected.value,
  disabled,
  state: dataState.value,
}));

type CheckboxGroupSetupExpose = Omit<
  CheckboxGroupExpose<Value>,
  keyof CheckboxGroupSlotState<Value> | "root"
> & {
  readonly [Key in keyof CheckboxGroupSlotState<Value>]: ComputedRef<
    CheckboxGroupSlotState<Value>[Key]
  >;
} & { readonly root: typeof root };

function field<Key extends keyof CheckboxGroupSlotState<Value>>(
  key: Key,
): ComputedRef<CheckboxGroupSlotState<Value>[Key]> {
  return computed(() => slotState.value[key]);
}

const exposed = {
  values: field("values"),
  options: field("options"),
  allSelected: field("allSelected"),
  someSelected: field("someSelected"),
  disabled: field("disabled"),
  state: field("state"),
  root,
  isSelected: (value: Value) => selectedIndexes.value.has(optionIndex(value)),
  setSelected: (value: Value, selected: boolean) => toggleIndex(optionIndex(value), selected),
  setAll: toggleAll,
  reset: state.reset,
} satisfies CheckboxGroupSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="root"
    role="group"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-disabled="disabled ? 'true' : undefined"
    part="root"
    data-vize-ui="checkbox-group"
    :data-state="dataState"
    :data-orientation="orientation"
    :data-disabled="disabled ? 'true' : undefined"
    :data-invalid="ariaInvalidValue === undefined ? undefined : 'true'"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
