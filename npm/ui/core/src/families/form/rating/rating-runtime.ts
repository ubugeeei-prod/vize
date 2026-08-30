import { computed, nextTick, watch, watchEffect } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import { useControllableState } from "../../../controllable-state.ts";
import { deriveDeterministicId, useDeterministicId } from "../../../deterministic-id.ts";
import {
  RATING_DEFAULT_COUNT,
  RATING_DEFAULT_MIN,
  getRatingState,
  normalizeRatingValue,
} from "./rating-state.ts";
import type {
  RatingExpose,
  RatingItemSlotState,
  RatingItemState,
  RatingProps,
  RatingStyle,
  RatingValue,
} from "./rating-types.ts";

type RatingRootRef = Readonly<ShallowRef<HTMLSpanElement | null>>;
type RatingRuntimeProps = {
  readonly [Key in keyof RatingProps]-?: RatingProps[Key] | undefined;
};

type RatingEmit = {
  (event: "update:modelValue", value: RatingValue): void;
  (event: "change", value: RatingValue, previous: RatingValue, nativeEvent: Event): void;
  (event: "clear", previous: number, nativeEvent: Event): void;
};

type RatingSetupExpose = Omit<
  RatingExpose,
  | "clearable"
  | "count"
  | "direction"
  | "disabled"
  | "elements"
  | "invalid"
  | "items"
  | "max"
  | "min"
  | "percent"
  | "readOnly"
  | "required"
  | "root"
  | "state"
  | "value"
> & {
  readonly clearable: ComputedRef<RatingExpose["clearable"]>;
  readonly count: ComputedRef<RatingExpose["count"]>;
  readonly direction: ComputedRef<RatingExpose["direction"]>;
  readonly disabled: ComputedRef<RatingExpose["disabled"]>;
  readonly elements: ComputedRef<RatingExpose["elements"]>;
  readonly invalid: ComputedRef<RatingExpose["invalid"]>;
  readonly items: ComputedRef<RatingExpose["items"]>;
  readonly max: ComputedRef<RatingExpose["max"]>;
  readonly min: ComputedRef<RatingExpose["min"]>;
  readonly percent: ComputedRef<RatingExpose["percent"]>;
  readonly readOnly: ComputedRef<RatingExpose["readOnly"]>;
  readonly required: ComputedRef<RatingExpose["required"]>;
  readonly root: RatingRootRef;
  readonly state: ComputedRef<RatingExpose["state"]>;
  readonly value: ComputedRef<RatingExpose["value"]>;
};

export function useRating(props: RatingRuntimeProps, emit: RatingEmit, root: RatingRootRef) {
  const controlId = useDeterministicId({ id: () => props.id, hint: "rating" });
  const ariaInvalidValue = computed(() => {
    if (props.ariaInvalid !== true && props.ariaInvalid !== "grammar") {
      return props.ariaInvalid === "spelling" ? "spelling" : undefined;
    }
    return props.ariaInvalid === true ? "true" : props.ariaInvalid;
  });
  const bounds = () => ({
    count: props.count ?? RATING_DEFAULT_COUNT,
    max: props.max,
    min: props.min ?? RATING_DEFAULT_MIN,
  });
  const valueState = useControllableState<RatingValue>({
    value: () =>
      props.modelValue === undefined ? undefined : normalizeRatingValue(props.modelValue, bounds()),
    defaultValue: () => normalizeRatingValue(props.defaultValue, bounds()),
    onChange: (value) => emit("update:modelValue", value),
  });
  const rating = computed(() =>
    getRatingState({
      ...bounds(),
      value: valueState.value.value,
      clearable: props.clearable === true,
      disabled: props.disabled === true,
      readOnly: props.readOnly === true,
      required: props.required === true,
      direction: props.dir,
      invalid: ariaInvalidValue.value !== undefined,
    }),
  );
  const currentValue = computed(() => rating.value.value);
  const minValue = computed(() => rating.value.min);
  const maxValue = computed(() => rating.value.max);
  const itemCount = computed(() => rating.value.count);
  const items = computed(() => rating.value.items);
  const percent = computed(() => rating.value.percent);
  const directionState = computed(() => rating.value.direction);
  const disabledState = computed(() => rating.value.disabled);
  const readOnlyState = computed(() => rating.value.readOnly);
  const requiredState = computed(() => rating.value.required);
  const invalidState = computed(() => rating.value.invalid);
  const clearableState = computed(() => rating.value.clearable);
  const dataState = computed(() => rating.value.state);
  const elements = computed<readonly HTMLInputElement[]>(() => {
    void items.value;
    return root.value === null
      ? []
      : [...root.value.querySelectorAll<HTMLInputElement>('[data-vize-ui="rating-control"]')];
  });
  const ratingStyle = computed<RatingStyle>(() => ({
    "--vize-rating-value": currentValue.value === null ? "" : String(currentValue.value),
    "--vize-rating-min": String(minValue.value),
    "--vize-rating-max": String(maxValue.value),
    "--vize-rating-count": String(itemCount.value),
    "--vize-rating-percent": `${percent.value}%`,
  }));
  const itemStates = computed<readonly RatingItemSlotState[]>(() =>
    items.value.map((itemValue, index): RatingItemSlotState => {
      const checked = currentValue.value === itemValue;
      const active = currentValue.value !== null && itemValue <= currentValue.value;
      const state: RatingItemState = disabledState.value
        ? "disabled"
        : readOnlyState.value
          ? "readonly"
          : checked
            ? "checked"
            : "unchecked";

      return {
        value: itemValue,
        index,
        currentValue: currentValue.value,
        checked,
        active,
        percent: ((index + 1) / itemCount.value) * 100,
        min: minValue.value,
        max: maxValue.value,
        count: itemCount.value,
        direction: directionState.value,
        disabled: disabledState.value,
        readOnly: readOnlyState.value,
        required: requiredState.value,
        invalid: invalidState.value,
        clearable: clearableState.value,
        state,
      };
    }),
  );
  let suppressedChangeValue: number | null = null;

  function syncNativeState(): void {
    for (const input of elements.value) input.checked = currentValue.value === Number(input.value);
  }

  watchEffect(syncNativeState);
  watch(
    root,
    (element, _previous, onCleanup) => {
      const form = element?.closest("form");
      if (form === undefined || form === null) return;
      const onReset = () => {
        if (!valueState.controlled.value) valueState.reset();
        void nextTick(syncNativeState);
      };
      form.addEventListener("reset", onReset);
      onCleanup(() => form.removeEventListener("reset", onReset));
    },
    { flush: "post", immediate: true },
  );

  function itemId(value: number): string {
    return deriveDeterministicId(controlId.value, `item-${value}`);
  }

  function itemAriaLabel(value: number): string {
    const prefix = (props.itemLabel ?? "Rating").trim();
    const suffix = `${value} of ${maxValue.value}`;
    return prefix.length === 0 ? suffix : `${prefix} ${suffix}`;
  }

  function commitUserValue(next: RatingValue, nativeEvent: Event): boolean {
    const previous = currentValue.value;
    const changed = valueState.set(next);
    if (!changed) return false;
    if (next === null && previous !== null) emit("clear", previous, nativeEvent);
    emit("change", next, previous, nativeEvent);
    return true;
  }

  function setValue(next: RatingValue): boolean {
    const changed = valueState.set(normalizeRatingValue(next, bounds()));
    void nextTick(syncNativeState);
    return changed;
  }

  function clear(): boolean {
    return setValue(null);
  }

  function focusValue(value: RatingValue, options?: FocusOptions): void {
    const target =
      value === null
        ? (elements.value.find((input) => !input.disabled) ?? null)
        : (elements.value.find((input) => Number(input.value) === value && !input.disabled) ??
          null);
    target?.focus(options);
  }

  function valueAtOffset(from: number, offset: number): number {
    const index = items.value.indexOf(from);
    const currentIndex = index === -1 ? 0 : index;
    const nextIndex = (currentIndex + offset + itemCount.value) % itemCount.value;
    return items.value[nextIndex] ?? minValue.value;
  }

  function keyboardValue(key: string, itemValue: number): RatingValue | undefined {
    if (key === "Home") return minValue.value;
    if (key === "End") return maxValue.value;
    if (clearableState.value && (key === "Backspace" || key === "Delete" || key === "Escape")) {
      return null;
    }
    if (key === " " || key === "Enter") {
      return clearableState.value && currentValue.value === itemValue ? null : itemValue;
    }

    const base = currentValue.value ?? itemValue;
    if (key === "ArrowDown") return valueAtOffset(base, 1);
    if (key === "ArrowUp") return valueAtOffset(base, -1);
    if (key === "ArrowRight") return valueAtOffset(base, directionState.value === "rtl" ? -1 : 1);
    if (key === "ArrowLeft") return valueAtOffset(base, directionState.value === "rtl" ? 1 : -1);
    return undefined;
  }

  function onItemClick(event: MouseEvent, itemValue: number): void {
    if (readOnlyState.value) {
      event.preventDefault();
      void nextTick(syncNativeState);
      return;
    }
    if (!clearableState.value || currentValue.value !== itemValue) return;

    event.preventDefault();
    suppressedChangeValue = itemValue;
    commitUserValue(null, event);
    void nextTick(() => {
      suppressedChangeValue = null;
      syncNativeState();
    });
  }

  function onItemChange(event: Event, itemValue: number): void {
    if (suppressedChangeValue === itemValue) {
      event.preventDefault();
      return;
    }
    if (readOnlyState.value) {
      event.preventDefault();
      void nextTick(syncNativeState);
      return;
    }
    if (!(event.currentTarget instanceof HTMLInputElement) || !event.currentTarget.checked) return;
    commitUserValue(itemValue, event);
    void nextTick(syncNativeState);
  }

  function onItemKeydown(event: KeyboardEvent, itemValue: number): void {
    const next = keyboardValue(event.key, itemValue);
    if (next === undefined) return;

    event.preventDefault();
    if (readOnlyState.value) {
      void nextTick(syncNativeState);
      return;
    }

    const changed = commitUserValue(next, event);
    if (changed && next !== null) void nextTick(() => focusValue(next));
    else void nextTick(syncNativeState);
  }

  const exposed = {
    clear,
    clearable: clearableState,
    count: itemCount,
    direction: directionState,
    disabled: disabledState,
    elements,
    focus: (options?: FocusOptions) => focusValue(currentValue.value, options),
    invalid: invalidState,
    items,
    max: maxValue,
    min: minValue,
    percent,
    readOnly: readOnlyState,
    required: requiredState,
    reset: valueState.reset,
    root,
    setValue,
    state: dataState,
    value: currentValue,
  } satisfies RatingSetupExpose;

  return {
    ariaInvalidValue,
    clearableState,
    controlId,
    currentValue,
    dataState,
    directionState,
    disabledState,
    exposed,
    invalidState,
    itemAriaLabel,
    itemCount,
    itemId,
    itemStates,
    items,
    maxValue,
    minValue,
    onItemChange,
    onItemClick,
    onItemKeydown,
    percent,
    readOnlyState,
    requiredState,
    intrinsicProps: computed(() => ({ style: ratingStyle.value })),
  };
}
