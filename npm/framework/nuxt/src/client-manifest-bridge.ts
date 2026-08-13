type CloseBundleHook = (this: unknown, ...args: unknown[]) => unknown;
type CloseBundleHookObject = { handler?: CloseBundleHook };

const VIZE_NUXT_CLIENT_MANIFEST_PATCHED = "__vizeNuxtClientManifestPatched";
const closeBundleByScope = new WeakMap<object, Promise<unknown>>();

export type NuxtClientManifestVitePlugin = {
  name?: string;
  closeBundle?: CloseBundleHook | CloseBundleHookObject;
  [VIZE_NUXT_CLIENT_MANIFEST_PATCHED]?: boolean;
};

/**
 * Nuxt's Environment API client-manifest plugin consumes and then removes the
 * Vite client manifest. Rolldown/Vite can invoke the SSR closeBundle path more
 * than once for a single build, so make that teardown idempotent per build.
 */
export function patchNuxtClientManifestCloseBundlePlugin(
  plugin: NuxtClientManifestVitePlugin | undefined,
  buildScope: object,
): void {
  if (!plugin || plugin.name !== "nuxt:client-manifest") {
    return;
  }
  if (plugin[VIZE_NUXT_CLIENT_MANIFEST_PATCHED]) {
    return;
  }

  if (typeof plugin.closeBundle === "function") {
    const original = plugin.closeBundle;
    plugin.closeBundle = function (this: unknown, ...args: unknown[]) {
      return runOnceForScope(buildScope, () => original.call(this, ...args));
    };
    plugin[VIZE_NUXT_CLIENT_MANIFEST_PATCHED] = true;
    return;
  }

  const closeBundle = plugin.closeBundle;
  if (!closeBundle || typeof closeBundle.handler !== "function") {
    return;
  }

  const original = closeBundle.handler;
  closeBundle.handler = function (this: unknown, ...args: unknown[]) {
    return runOnceForScope(buildScope, () => original.call(this, ...args));
  };
  plugin[VIZE_NUXT_CLIENT_MANIFEST_PATCHED] = true;
}

function runOnceForScope(buildScope: object, run: () => unknown): Promise<unknown> {
  const pending = closeBundleByScope.get(buildScope);
  if (pending) {
    return pending;
  }

  let next: Promise<unknown>;
  try {
    next = Promise.resolve(run());
  } catch (error) {
    next = Promise.reject(error);
  }
  // A failed teardown must stay retryable, so only successful runs stay cached.
  const tracked: Promise<unknown> = next.catch((error: unknown) => {
    if (closeBundleByScope.get(buildScope) === tracked) {
      closeBundleByScope.delete(buildScope);
    }
    throw error;
  });
  closeBundleByScope.set(buildScope, tracked);
  return tracked;
}
