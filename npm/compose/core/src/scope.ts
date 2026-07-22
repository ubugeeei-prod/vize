import { getCurrentScope, onScopeDispose } from "vue";

/**
 * Register cleanup in the active reactive scope when one exists.
 *
 * This is the shared lifecycle primitive of the package: composables hand
 * their teardown here so owned resources are released when the surrounding
 * scope stops.
 *
 * Never throws and is safe during server rendering; no browser globals are
 * read. When no scope is active the cleanup is not registered and disposal
 * ownership stays with the caller.
 *
 * @param cleanup Cleanup invoked exactly once when the scope is disposed.
 * @returns Whether the cleanup was registered.
 */
export function tryOnScopeDispose(cleanup: () => void): boolean {
  if (!getCurrentScope()) return false;
  onScopeDispose(cleanup);
  return true;
}
