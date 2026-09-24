import { inject, provide } from "vue";
import type { InjectionKey } from "vue";

/** Options for {@link createInjectionState}. */
export interface CreateInjectionStateOptions<State> {
  /**
   * Key used for `provide`/`inject`.
   *
   * @default a fresh unique symbol
   */
  readonly injectionKey?: InjectionKey<State>;

  /**
   * Value injected when no ancestor provides the state; also narrows the
   * injector's return type to `State`.
   *
   * @default undefined
   */
  readonly defaultValue?: State;
}

/** Provider, injector, and key returned by {@link createInjectionState}. */
export type InjectionState<Arguments extends readonly unknown[], State, Injected> = readonly [
  useProvidingState: (...args: Arguments) => State,
  useInjectedState: () => Injected,
  injectionKey: InjectionKey<State>,
];

/**
 * Create a typed provide/inject pair around a composable, for component
 * subtrees that share request-local state.
 *
 * The provider runs the composable with its arguments, provides the result
 * under a typed `InjectionKey`, and returns it. The injector returns the
 * nearest provided state, or `defaultValue`, or `undefined` (reflected in
 * its return type). State lives in the component tree, so unlike module
 * globals it is naturally per request during SSR. Both must be called
 * during `setup` (or inside `app.runWithContext`).
 *
 * @example
 * ```ts
 * const [useProvideCounter, useCounter] = createInjectionState((initial: number) => {
 *   const count = ref(initial);
 *   return { count, increment: () => count.value++ };
 * });
 * ```
 *
 * @param composable Creates the state in the providing component.
 * @param options Custom key and default value.
 * @default options {}
 * @returns `[useProvidingState, useInjectedState, injectionKey]`.
 */
export function createInjectionState<Arguments extends readonly unknown[], State>(
  composable: (...args: Arguments) => State,
  options: CreateInjectionStateOptions<State> & { readonly defaultValue: State },
): InjectionState<Arguments, State, State>;
export function createInjectionState<Arguments extends readonly unknown[], State>(
  composable: (...args: Arguments) => State,
  options?: CreateInjectionStateOptions<State>,
): InjectionState<Arguments, State, State | undefined>;
export function createInjectionState<Arguments extends readonly unknown[], State>(
  composable: (...args: Arguments) => State,
  options: CreateInjectionStateOptions<State> = {},
): InjectionState<Arguments, State, State | undefined> {
  const generatedKey: InjectionKey<State> = Symbol("InjectionState");
  const key = options.injectionKey ?? generatedKey;
  const useProvidingState = (...args: Arguments): State => {
    const state = composable(...args);
    provide(key, state);
    return state;
  };
  const useInjectedState = (): State | undefined =>
    options.defaultValue === undefined ? inject(key) : inject(key, options.defaultValue);
  return [useProvidingState, useInjectedState, key];
}
