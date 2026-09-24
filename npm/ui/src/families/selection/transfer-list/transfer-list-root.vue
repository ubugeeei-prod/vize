<script setup lang="ts" generic="T">
import { computed, shallowRef } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { transferListContext } from "./transfer-list-context.ts";
import type { TransferListContextValue, TransferListSideState } from "./transfer-list-context.ts";
import {
  containsTransferFilter,
  createTransferEquality,
  defaultTransferText,
  serializeTransferValue,
  transferItems,
} from "./transfer-list-model.ts";
import type {
  TransferListAction,
  TransferListDirection,
  TransferListSide,
} from "./transfer-list-model.ts";
import type {
  TransferListRootExpose,
  TransferListRootProps,
  TransferListSlotState,
} from "./transfer-list-types.ts";

const {
  id = undefined,
  items,
  modelValue = undefined,
  defaultValue = undefined,
  by = undefined,
  itemText = undefined,
  itemDisabled = undefined,
  filter = undefined,
  sourceQuery = undefined,
  targetQuery = undefined,
  max = Number.POSITIVE_INFINITY,
  orderMode = "append",
  disabled = false,
  name = undefined,
  form = undefined,
  formValue = undefined,
} = defineProps<TransferListRootProps<T>>();

const emit = defineEmits<{
  /** Fired when the target list requests a new controlled value. */
  "update:modelValue": [value: readonly T[]];

  /** Fired after a move: next target, previous target, moved items, direction, and the triggering event. */
  change: [
    value: readonly T[],
    previous: readonly T[],
    moved: readonly T[],
    direction: TransferListDirection,
    nativeEvent: Event | null,
  ];

  /** Fired when the source search text requests a new controlled value. */
  "update:sourceQuery": [query: string];

  /** Fired when the target search text requests a new controlled value. */
  "update:targetQuery": [query: string];
}>();

defineSlots<{
  /** Panels, search fields, and action buttons. Receives both sides and capacity. */
  default?(props: TransferListSlotState<T>): unknown;
}>();

/** One hidden form input. */
interface FormEntry {
  readonly key: string;
  readonly value: string;
}

const baseId = useDeterministicId({ id: () => id, hint: "transfer-list" });
const disabledState = computed(() => disabled);
const equals = computed(() => createTransferEquality<T>(by));
const targetState = useControllableState<readonly T[]>({
  value: () => modelValue,
  defaultValue: () => Object.freeze([...(defaultValue ?? [])]),
  equals: (left, right) =>
    left.length === right.length &&
    left.every((value, index) => equals.value(value, right[index] as T)),
  onChange: (next) => emit("update:modelValue", next),
});
const sourceQueryState = useControllableState<string>({
  value: () => sourceQuery,
  defaultValue: "",
  onChange: (next) => emit("update:sourceQuery", next),
});
const targetQueryState = useControllableState<string>({
  value: () => targetQuery,
  defaultValue: "",
  onChange: (next) => emit("update:targetQuery", next),
});
const checkedSource = shallowRef<readonly T[]>([]);
const checkedTarget = shallowRef<readonly T[]>([]);
const panelFocus = new Map<TransferListSide, () => void>();

function includes(values: readonly T[], value: T): boolean {
  return values.some((candidate) => equals.value(candidate, value));
}

function textOf(value: T): string {
  return itemText?.(value) ?? defaultTransferText(value);
}

function isItemDisabled(value: T): boolean {
  return disabledState.value || itemDisabled?.(value) === true;
}

function applyFilter(values: readonly T[], query: string): readonly T[] {
  if (query.length === 0) return values;
  const match = filter ?? containsTransferFilter;
  return values.filter((value) => match(value, query, textOf(value)));
}

const target = computed(() => targetState.value.value);
const source = computed(() => items.filter((item) => !includes(target.value, item)));
const full = computed(() => target.value.length >= max);

function sideState(
  all: ComputedRef<readonly T[]>,
  query: ComputedRef<string>,
  checked: typeof checkedSource,
): TransferListSideState<T> {
  const visible = computed(() => applyFilter(all.value, query.value));
  return {
    all,
    checked: computed(() => checked.value.filter((value) => includes(all.value, value))),
    query,
    visible,
  };
}

const sides: Record<TransferListSide, TransferListSideState<T>> = {
  source: sideState(source, sourceQueryState.value, checkedSource),
  target: sideState(target, targetQueryState.value, checkedTarget),
};

function checkedRef(side: TransferListSide): typeof checkedSource {
  return side === "source" ? checkedSource : checkedTarget;
}

function currentTarget(): readonly T[] {
  return targetState.value.value;
}

function moveValues(
  side: TransferListSide,
  values: readonly T[],
  event: Event | null,
): readonly T[] {
  if (disabledState.value) return [];
  const direction: TransferListDirection = side === "source" ? "to-target" : "to-source";
  const previous = currentTarget();
  const result = transferItems({
    direction,
    equals: equals.value,
    items,
    max,
    moving: values.filter((value) => !isItemDisabled(value)),
    orderMode,
    target: previous,
  });
  if (result.moved.length === 0) return result.moved;
  targetState.set(result.target);
  const checked = checkedRef(side);
  checked.value = checked.value.filter((value) => !includes(result.moved, value));
  emit("change", result.target, previous, result.moved, direction, event);
  return result.moved;
}

function candidates(action: TransferListAction): {
  readonly side: TransferListSide;
  readonly values: readonly T[];
} {
  const side: TransferListSide = action.endsWith("to-target") ? "source" : "target";
  const state = sides[side];
  const values = action.startsWith("move-all") ? state.visible.value : state.checked.value;
  return { side, values: values.filter((value) => !isItemDisabled(value)) };
}

function canRun(action: TransferListAction): boolean {
  if (disabledState.value) return false;
  const { side, values } = candidates(action);
  if (values.length === 0) return false;
  return side === "target" || !full.value;
}

function run(action: TransferListAction, event: Event | null): readonly T[] {
  const { side, values } = candidates(action);
  return moveValues(side, values, event);
}

const formEntries = computed<readonly FormEntry[]>(() =>
  target.value.map((value, index) => ({
    key: String(index),
    value: formValue?.(value) ?? serializeTransferValue(value, by),
  })),
);
const slotState = computed<TransferListSlotState<T>>(() => ({
  checkedSource: sides.source.checked.value,
  checkedTarget: sides.target.checked.value,
  visibleSource: sides.source.visible.value,
  visibleTarget: sides.target.visible.value,
  disabled: disabledState.value,
  full: full.value,
  source: source.value,
  target: target.value,
}));

const context: TransferListContextValue<T> = {
  baseId,
  canRun,
  disabled: disabledState,
  focusPanel: (side) => panelFocus.get(side)?.(),
  full,
  isChecked: (side, value) => includes(sides[side].checked.value, value),
  isItemDisabled,
  moveValues,
  panelId: (side) => deriveDeterministicId(baseId.value, `${side}-listbox`),
  registerPanel: (side, focus) => {
    panelFocus.set(side, focus);
    return () => {
      if (panelFocus.get(side) === focus) panelFocus.delete(side);
    };
  },
  run,
  setChecked: (side, values) => {
    checkedRef(side).value = Object.freeze(values.filter((value) => !isItemDisabled(value)));
  },
  setQuery: (side, query) => {
    if (side === "source") sourceQueryState.set(query);
    else targetQueryState.set(query);
  },
  side: (side) => sides[side],
  textOf,
  toggleChecked: (side, value) => {
    if (isItemDisabled(value)) return;
    const checked = checkedRef(side);
    checked.value = includes(checked.value, value)
      ? checked.value.filter((candidate) => !equals.value(candidate, value))
      : Object.freeze([...checked.value, value]);
  },
};
transferListContext.provide(context);

type TransferListRootSetupExpose = Omit<TransferListRootExpose<T>, "target"> & {
  readonly target: ComputedRef<readonly T[]>;
};

const exposed = {
  moveToSource: () => run("move-selected-to-source", null),
  moveToTarget: () => run("move-selected-to-target", null),
  reset: () => targetState.reset(),
  target,
} satisfies TransferListRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="transfer-list"
    part="root"
    :data-full="full ? 'true' : undefined"
    :data-disabled="disabledState ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
    <template v-if="name !== undefined">
      <input
        v-for="entry in formEntries as readonly FormEntry[]"
        :key="entry.key"
        type="hidden"
        data-vize-ui="transfer-list-native"
        :name
        :form
        :value="entry.value"
        :disabled="disabledState"
      />
    </template>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
