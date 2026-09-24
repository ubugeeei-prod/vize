import { effectScope } from "vue";

/**
 * Lift a state factory into application-wide state shared by every caller.
 *
 * The factory runs once, lazily on the first call, inside a detached effect
 * scope so its watchers and computeds outlive any component; every later
 * call returns the same state. Because the state is module-level, it is
 * shared across all server requests: on the server, keep per-request state
 * in `provide`/`createInjectionState` instead, or reset it per request.
 *
 * @example
 * ```ts
 * const useSession = createGlobalState(() => ({ user: shallowRef<User | null>(null) }));
 * useSession().user.value = currentUser;
 * ```
 *
 * @param factory Creates the state; may use any composable.
 * @throws `Error` tagged `VIZE_COMPOSE_GLOBAL_STATE_INACTIVE_SCOPE` if the
 * private scope cannot run (never expected in practice).
 * @returns Accessor returning the shared state.
 */
export function createGlobalState<State>(factory: () => State): () => State {
  let created: { readonly state: State } | undefined;
  return () => {
    if (created === undefined) {
      created = effectScope(true).run(() => ({ state: factory() }));
      if (created === undefined) {
        throw new Error("[VIZE_COMPOSE_GLOBAL_STATE_INACTIVE_SCOPE] the state scope did not run");
      }
    }
    return created.state;
  };
}
