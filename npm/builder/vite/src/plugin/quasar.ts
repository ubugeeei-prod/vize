const bridgePatched = Symbol("vize.quasarBridgePatched");

type QuasarLikePlugin = {
  name?: string;
  transform?: (...args: unknown[]) => unknown;
  [bridgePatched]?: boolean;
};

const VIZE_SSR_PREFIX = "\0vize-ssr:";
const LEGACY_VIZE_PREFIX = "\0vize:";
const plainSsrPrefix = VIZE_SSR_PREFIX.slice(1);
const plainLegacyPrefix = LEGACY_VIZE_PREFIX.slice(1);

function stripBridgePrefix(id: string): string {
  if (id.startsWith(VIZE_SSR_PREFIX)) {
    return id.slice(VIZE_SSR_PREFIX.length);
  }
  if (id.startsWith(LEGACY_VIZE_PREFIX)) {
    return id.slice(LEGACY_VIZE_PREFIX.length);
  }
  if (id.startsWith(plainSsrPrefix)) {
    return id.slice(plainSsrPrefix.length);
  }
  if (id.startsWith(plainLegacyPrefix)) {
    return id.slice(plainLegacyPrefix.length);
  }
  if (id.startsWith("\0")) {
    return id.slice(1);
  }
  return id;
}

function isQuasarBridgeModuleId(id: string): boolean {
  return /\.vue\.ts(?:[?#]|$)/.test(stripBridgePrefix(id));
}

function normalizeQuasarBridgeModuleId(id: string): string {
  return stripBridgePrefix(id).replace(/\.vue\.ts(?=[?#]|$)/, ".vue");
}

export function patchQuasarBridge(plugins: QuasarLikePlugin[]): void {
  for (const plugin of plugins) {
    if (
      plugin.name !== "vite:quasar:script" ||
      typeof plugin.transform !== "function" ||
      plugin[bridgePatched]
    ) {
      continue;
    }

    const originalTransform = plugin.transform;

    plugin.transform = function (
      this: unknown,
      code: string,
      id: string,
      ...args: unknown[]
    ): unknown {
      if (!isQuasarBridgeModuleId(id)) {
        return originalTransform.call(this, code, id, ...args);
      }

      return originalTransform.call(this, code, normalizeQuasarBridgeModuleId(id), ...args);
    };

    plugin[bridgePatched] = true;
  }
}
