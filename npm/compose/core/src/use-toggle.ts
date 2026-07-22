import { shallowRef } from "vue";
import type { Ref } from "vue";

/** Reactive controls returned by {@link useToggle}. */
export interface ToggleControls {
  /**
   * Owned boolean state.
   *
   * Deliberately writable: unlike the derived views elsewhere in this
   * package, the toggle owns its state, so assigning the ref directly (for
   * example through `v-model`) is equivalent to calling
   * {@link ToggleControls.toggle} with a forced value.
   */
  readonly state: Ref<boolean>;

  /**
   * Invert the state, or force it to `force` when the argument is given.
   * Passing an explicit `undefined` behaves like passing no argument.
   *
   * @param force Value assigned instead of inverting.
   * @returns The state after the change.
   */
  readonly toggle: (force?: boolean) => boolean;
}

/**
 * Create owned boolean state with an inverting control.
 *
 * Purely synchronous state: safe during server rendering (no browser
 * globals, no timers) and nothing to dispose, so it works inside and
 * outside reactive scopes alike.
 *
 * @example
 * ```ts
 * const { state: open, toggle } = useToggle();
 * toggle(); // true
 * toggle(false); // false
 * open.value; // false
 * ```
 *
 * @param initial State before the first toggle.
 * @default initial false
 * @returns The writable state and its toggle control.
 */
export function useToggle(initial = false): ToggleControls {
  const state = shallowRef(initial);

  const toggle = (force?: boolean): boolean => {
    state.value = force ?? !state.value;
    return state.value;
  };

  return { state, toggle };
}
