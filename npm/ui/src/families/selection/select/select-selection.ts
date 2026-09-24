import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import type { ControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { areSelectListsEqual, fromSelectList, toSelectList } from "./select-model.ts";
import type { SelectModelValue, SelectValueEquality } from "./select-model.ts";

/** Options for {@link useSelectSelection}. */
export interface SelectSelectionOptions<T, Multiple extends boolean> {
  /** Controlled public model; `undefined` selects uncontrolled behavior. */
  readonly modelValue: () => SelectModelValue<T, Multiple> | undefined;

  /** Public default model restored by reset. */
  readonly defaultValue: () => SelectModelValue<T, Multiple> | undefined;

  /** Whether the public model is an array. */
  readonly multiple: () => boolean;

  /** Value equality derived from `by`. */
  readonly equals: () => SelectValueEquality<T>;

  /** Called with the public model after a distinct selection request. */
  readonly onChange: (value: SelectModelValue<T, Multiple>) => void;
}

/**
 * Controlled/uncontrolled selection shared by Select and Combobox roots.
 *
 * The state is held as a normalized readonly list; the public `T | null` or
 * `readonly T[]` model is produced only when notifying the consumer.
 */
export function useSelectSelection<T, Multiple extends boolean>(
  options: SelectSelectionOptions<T, Multiple>,
): ControllableState<readonly T[]> {
  return useControllableState<readonly T[]>({
    value: () => {
      const model = options.modelValue();
      return model === undefined
        ? undefined
        : toSelectList<T>(model, options.multiple(), options.equals());
    },
    defaultValue: () =>
      toSelectList<T>(options.defaultValue(), options.multiple(), options.equals()),
    equals: (left, right) => areSelectListsEqual(left, right, options.equals()),
    onChange: (next) => options.onChange(fromSelectList<T, Multiple>(next, options.multiple())),
  });
}
