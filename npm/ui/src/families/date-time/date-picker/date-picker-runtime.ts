import { computed } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { isSameDay, normalizePlainDate } from "../calendar/plain-date.ts";
import type { PlainDate } from "../calendar/plain-date.ts";
import type { PickerOptions, PickerSharedContext } from "./date-picker-context.ts";
import type { PickerOpenState } from "./date-picker-types.ts";

/** Inputs of {@link usePickerRoot}. */
export interface PickerRootInput {
  readonly id: () => string | null | undefined;
  readonly hint: string;
  readonly open: () => boolean | undefined;
  readonly defaultOpen: () => boolean;
  readonly disabled: () => boolean;
  readonly readOnly: () => boolean;
  readonly required: () => boolean;
  readonly options: () => PickerOptions;
  readonly onOpenChange: (value: boolean, previous: boolean, event: Event | null) => void;
}

/**
 * Open state, ids, forwarded options, and the initial-focus registry shared by
 * DatePickerRoot and DateRangePickerRoot.
 */
export function usePickerRoot(input: PickerRootInput) {
  const baseId = useDeterministicId({ id: input.id, hint: input.hint });
  const popoverId = computed(() => deriveDeterministicId(baseId.value, "popover"));
  const disabled = computed(() => input.disabled());
  const readOnly = computed(() => input.readOnly());
  const required = computed(() => input.required());
  const options = computed(() => input.options());
  const openState = useControllableState<boolean>({
    value: input.open,
    defaultValue: input.defaultOpen,
  });
  const open = computed(() => openState.value.value && !disabled.value);
  const state = computed<PickerOpenState>(() => (open.value ? "open" : "closed"));
  let focusTarget: (() => HTMLElement | null) | null = null;

  function setOpen(value: boolean, event: Event | null = null): boolean {
    const previous = open.value;
    const next = value && !disabled.value;
    if (previous === next) return false;
    openState.set(next);
    input.onOpenChange(next, previous, event);
    return true;
  }

  const shared: PickerSharedContext = {
    open,
    setOpen,
    options,
    disabled,
    readOnly,
    required,
    registerFocusTarget: (target) => {
      focusTarget = target;
      return () => {
        if (focusTarget === target) focusTarget = null;
      };
    },
    focusTarget: () => focusTarget?.() ?? null,
  };

  return {
    baseId,
    popoverId,
    state,
    shared,
    onPopoverOpenChange: (value: boolean, _previous: boolean, event: Event | null) => {
      setOpen(value, event);
    },
  };
}

/** Inputs of {@link usePickerValue}. */
export interface PickerValueInput {
  readonly value: () => PlainDate | null | undefined;
  readonly defaultValue: () => PlainDate | null | undefined;
  readonly locked: () => boolean;
  readonly onUpdate: (value: PlainDate | null) => void;
  readonly onChange: (
    value: PlainDate | null,
    previous: PlainDate | null,
    event: Event | null,
  ) => void;
}

/** Controlled/uncontrolled single date shared by DatePicker parts. */
export function usePickerValue(input: PickerValueInput) {
  const state = useControllableState<PlainDate | null>({
    value: () => {
      const value = input.value();
      return value === undefined ? undefined : normalizePlainDate(value);
    },
    defaultValue: () => normalizePlainDate(input.defaultValue()),
    equals: isSameDay,
    onChange: (value) => input.onUpdate(value),
  });
  const value = computed(() => state.value.value);

  function setValue(next: PlainDate | null, event: Event | null = null): boolean {
    if (input.locked()) return false;
    const normalized = normalizePlainDate(next);
    const previous = value.value;
    const changed = state.set(normalized);
    if (changed) input.onChange(normalized, previous, event);
    return changed;
  }

  return { value, setValue };
}
