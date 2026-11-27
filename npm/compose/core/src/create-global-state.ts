import { effectScope } from "vue";

/** Options for {@link createGlobalState}. */
export interface CreateGlobalStateOptions {
  /**
   * Share one state across callers on the server too. Disabled by default
   * because module-level sharing leaks state between requests; without a
   * browser `window` every call then gets its own state.
   *
   * @default false
   */
  readonly shareOnServer?: boolean;
}

/**
 * Lift a state factory into application-wide state shared by every caller.
 *
 * The factory runs once, lazily on the first call, inside a detached effect
 * scope so its watchers and computeds outlive any component; every later
 * call returns the same state. On the server every call runs the factory
 * afresh in the caller's scope unless `shareOnServer` is set, so request
 * state never leaks; keep per-request state in
 * `provide`/`createInjectionState` instead.
 *
 * @example
 * ```ts
 * const useCommandPalette = createGlobalState(() => ({ open: shallowRef(false) }));
 * useCommandPalette().open.value = true;
 * ```
 *
 * @param factory Creates the state; may use any composable.
 * @param options Server sharing policy.
 * @default options {}
 * @throws `Error` tagged `VIZE_COMPOSE_GLOBAL_STATE_INACTIVE_SCOPE` if the
 * private scope cannot run (never expected in practice).
 * @returns Accessor returning the shared state.
 */
export function createGlobalState<State>(
  factory: () => State,
  options: CreateGlobalStateOptions = {},
): () => State {
  let created: { readonly state: State } | undefined;
  return () => {
    if (typeof window === "undefined" && !(options.shareOnServer ?? false)) return factory();
    if (created === undefined) {
      created = effectScope(true).run(() => ({ state: factory() }));
      if (created === undefined) {
        throw new Error("[VIZE_COMPOSE_GLOBAL_STATE_INACTIVE_SCOPE] the state scope did not run");
      }
    }
    return created.state;
  };
}
