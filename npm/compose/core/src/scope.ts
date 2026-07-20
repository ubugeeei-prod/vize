import { getCurrentScope, onScopeDispose } from "vue";

/**
 * Register cleanup in the active reactive scope when one exists.
 *
 * @param cleanup Cleanup invoked exactly once when the scope is disposed.
 * @returns Whether the cleanup was registered.
 */
export function tryOnScopeDispose(cleanup: () => void): boolean {
  if (!getCurrentScope()) return false;
  onScopeDispose(cleanup);
  return true;
}
