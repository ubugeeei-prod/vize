type CloseBundleHook = (this: unknown, ...args: unknown[]) => unknown;
type CloseBundleHookObject = { handler?: CloseBundleHook };
/** An absent environment (a hook called outside the Environment API) keys as undefined. */
type EnvironmentName = string | undefined;

const VIZE_NUXT_CLIENT_MANIFEST_PATCHED = "__vizeNuxtClientManifestPatched";
const closeBundleByScope = new WeakMap<object, Map<EnvironmentName, Promise<unknown>>>();

export type NuxtClientManifestVitePlugin = {
  name?: string;
  closeBundle?: CloseBundleHook | CloseBundleHookObject;
  [VIZE_NUXT_CLIENT_MANIFEST_PATCHED]?: boolean;
};

/**
 * Nuxt's Environment API client-manifest plugin consumes and then removes the
 * Vite client manifest. Rolldown/Vite can invoke the SSR closeBundle path more
 * than once for a single build, so make that teardown idempotent per build and
 * environment. The hook runs once for every environment it applies to and only
 * the SSR call finalizes the manifest, so an earlier client call must not
 * satisfy the SSR one.
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
      return runOnceForEnvironment(buildScope, environmentName(this), () =>
        original.call(this, ...args),
      );
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
    return runOnceForEnvironment(buildScope, environmentName(this), () =>
      original.call(this, ...args),
    );
  };
  plugin[VIZE_NUXT_CLIENT_MANIFEST_PATCHED] = true;
}

function environmentName(context: unknown): EnvironmentName {
  if (!context || typeof context !== "object" || !("environment" in context)) return undefined;
  const environment = context.environment;
  if (!environment || typeof environment !== "object" || !("name" in environment)) {
    return undefined;
  }
  return typeof environment.name === "string" ? environment.name : undefined;
}

function runOnceForEnvironment(
  buildScope: object,
  environment: EnvironmentName,
  run: () => unknown,
): Promise<unknown> {
  let runs = closeBundleByScope.get(buildScope);
  if (!runs) {
    runs = new Map();
    closeBundleByScope.set(buildScope, runs);
  }
  const pending = runs.get(environment);
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
    if (runs.get(environment) === tracked) {
      runs.delete(environment);
    }
    throw error;
  });
  runs.set(environment, tracked);
  return tracked;
}
