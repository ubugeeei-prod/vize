import { effectScope } from "vue";
import type { EffectScope } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Options for {@link createSharedComposable}. */
export interface CreateSharedComposableOptions {
  /**
   * Share one instance across callers on the server too. Disabled by
   * default because module-level sharing leaks state between requests;
   * without a browser `window` every call then gets its own instance.
   *
   * @default false
   */
  readonly shareOnServer?: boolean;
}

/**
 * Share one instance of a composable between all simultaneously mounted
 * callers, reference-counted by their effect scopes.
 *
 * The first caller creates the instance inside a detached scope using its
 * arguments (later callers' arguments are ignored while it lives). When the
 * last subscribing scope stops, the instance's scope is stopped and the next
 * call starts fresh. Calls outside any effect scope subscribe permanently.
 * On the server every call gets its own instance unless `shareOnServer` is
 * set, so request state never leaks.
 *
 * @example
 * ```ts
 * const useSharedMouse = createSharedComposable(useMouse);
 * ```
 *
 * @param composable Composable to share.
 * @param options Server sharing policy.
 * @default options {}
 * @throws `Error` tagged `VIZE_COMPOSE_SHARED_COMPOSABLE_INACTIVE_SCOPE` if
 * the private scope cannot run (never expected in practice).
 * @returns A composable with the same signature that shares its result.
 */
export function createSharedComposable<Arguments extends readonly unknown[], Result>(
  composable: (...args: Arguments) => Result,
  options: CreateSharedComposableOptions = {},
): (...args: Arguments) => Result {
  let subscribers = 0;
  let shared: { readonly result: Result; readonly scope: EffectScope } | undefined;

  const release = (): void => {
    subscribers -= 1;
    if (subscribers > 0 || shared === undefined) return;
    shared.scope.stop();
    shared = undefined;
  };

  return (...args) => {
    if (typeof window === "undefined" && !(options.shareOnServer ?? false)) {
      return composable(...args);
    }
    subscribers += 1;
    if (shared === undefined) {
      const scope = effectScope(true);
      const result = scope.run(() => ({ value: composable(...args) }));
      if (result === undefined) {
        throw new Error(
          "[VIZE_COMPOSE_SHARED_COMPOSABLE_INACTIVE_SCOPE] the shared scope did not run",
        );
      }
      shared = { result: result.value, scope };
    }
    const { result } = shared;
    tryOnScopeDispose(release);
    return result;
  };
}
