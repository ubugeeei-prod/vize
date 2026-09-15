import type {
  ControllableState,
  ControllableStateOptions,
  ControlledStateOptions,
  StateValueUpdater,
  UncontrolledStateOptions,
} from "./types.ts";

export function createControllableState<State>(
  options: ControlledStateOptions<State>,
): ControllableState<State, true>;
export function createControllableState<State>(
  options: UncontrolledStateOptions<State>,
): ControllableState<State, false>;
export function createControllableState<State>(
  options: ControllableStateOptions<State>,
): ControllableState<State, boolean> {
  const controlled = "value" in options;
  const equals = options.equals ?? Object.is;
  let uncontrolledValue = controlled ? undefined : options.defaultValue;

  const read = () => (controlled ? options.value() : (uncontrolledValue as State));
  const update = (next: StateValueUpdater<State>) => {
    const previous = read();
    const value =
      typeof next === "function" ? (next as (state: Readonly<State>) => State)(previous) : next;
    if (equals(previous, value)) return previous;
    options.onChange?.(value, { previous, next: value, controlled });
    if (!controlled) uncontrolledValue = value;
    return value;
  };

  return {
    controlled,
    get value() {
      return read();
    },
    set: update,
    reset() {
      if (controlled) return read();
      return update((options as UncontrolledStateOptions<State>).defaultValue);
    },
    snapshot: read,
  };
}
