import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

/** Parameter list of a reactified function: every argument may be a ref or getter. */
export type ReactifiedArguments<Arguments extends readonly unknown[]> = {
  readonly [Index in keyof Arguments]: MaybeRefOrGetter<Arguments[Index]>;
};

/** Function returned by {@link reactify}. */
export type Reactified<Arguments extends readonly unknown[], Result> = (
  ...args: ReactifiedArguments<Arguments>
) => ComputedRef<Result>;

/**
 * Turn a plain function into one that accepts refs/getters and returns a
 * computed result.
 *
 * Arity and parameter names are preserved; each parameter additionally
 * accepts `Ref<T>` or `() => T`. The computed re-runs the original function
 * whenever a reactive argument changes. Pure derived state: SSR-safe and
 * nothing to dispose.
 *
 * @example
 * ```ts
 * const add = reactify((left: number, right: number) => left + right);
 * const sum = add(count, 1); // ComputedRef<number>
 * ```
 *
 * @param fn Plain function to lift.
 * @returns The reactified function.
 */
export function reactify<Arguments extends readonly unknown[], Result>(
  fn: (...args: Arguments) => Result,
): Reactified<Arguments, Result>;
export function reactify(
  fn: (...args: readonly unknown[]) => unknown,
): (...args: readonly MaybeRefOrGetter<unknown>[]) => ComputedRef<unknown> {
  return (...args) => computed(() => fn(...args.map((argument) => toValue(argument))));
}
