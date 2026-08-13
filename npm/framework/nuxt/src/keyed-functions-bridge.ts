import {
  isVizeVirtualVueModuleId,
  normalizeNuxtInjectedKeysForVizeVirtualModule,
} from "./utils.ts";

export type ViteTransformResult = string | { code?: string; map?: unknown } | null | undefined;

function normalizeNuxtKeyedTransformResult(
  id: string,
  result: ViteTransformResult,
): ViteTransformResult {
  if (!isVizeVirtualVueModuleId(id) || result == null) {
    return result;
  }
  if (typeof result === "string") {
    return normalizeNuxtInjectedKeysForVizeVirtualModule(result, id);
  }
  if (typeof result.code !== "string") {
    return result;
  }
  const code = normalizeNuxtInjectedKeysForVizeVirtualModule(result.code, id);
  return code === result.code ? result : { ...result, code };
}

/**
 * Nuxt's `nuxt:compiler:keyed-functions` plugin rewrites injected keys, but it
 * skips Vize's \0-prefixed virtual modules. Re-run the normalization on the
 * plugin result so keys stay stable for Vize-compiled SFCs.
 */
export function patchNuxtKeyedFunctionsPlugin(plugin: { transform?: unknown }): void {
  if (typeof plugin.transform === "function") {
    const original = plugin.transform;
    plugin.transform = async function (
      this: unknown,
      code: string,
      id: string,
      ...args: unknown[]
    ) {
      const result = (await original.call(this, code, id, ...args)) as ViteTransformResult;
      return normalizeNuxtKeyedTransformResult(id, result);
    };
    return;
  }

  const transform = plugin.transform as { handler?: unknown } | undefined;
  if (!transform || typeof transform.handler !== "function") {
    return;
  }

  const original = transform.handler;
  transform.handler = async function (this: unknown, code: string, id: string, ...args: unknown[]) {
    const result = (await original.call(this, code, id, ...args)) as ViteTransformResult;
    return normalizeNuxtKeyedTransformResult(id, result);
  };
}
