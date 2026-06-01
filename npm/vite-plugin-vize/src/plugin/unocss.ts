import fs from "node:fs";

type UnoCssLikePlugin = {
  name?: string;
  transform?: (...args: unknown[]) => unknown;
  [bridgePatched]?: boolean;
};

const bridgePatched = Symbol("vize.unocssBridgePatched");

const VIZE_SSR_PREFIX = "\0vize-ssr:";
const plainSsrPrefix = VIZE_SSR_PREFIX.slice(1);
export const MAX_UNOCSS_ORIGINAL_SOURCE_BYTES = 2 * 1024 * 1024;

function stripBridgePrefix(id: string): string {
  if (id.startsWith(VIZE_SSR_PREFIX)) {
    return id.slice(VIZE_SSR_PREFIX.length);
  }
  if (id.startsWith(plainSsrPrefix)) {
    return id.slice(plainSsrPrefix.length);
  }
  if (id.startsWith("\0")) {
    return id.slice(1);
  }
  return id;
}

function isUnoCssBridgeModuleId(id: string): boolean {
  return /\.vue\.ts(?:\?|$)/.test(stripBridgePrefix(id));
}

function normalizeUnoCssBridgeModuleId(id: string): string {
  return stripBridgePrefix(id).replace(/\.ts(?=\?|$)/, "");
}

function appendOriginalVueSourceForUnoCss(code: string, normalizedId: string): string {
  const sourcePath = normalizedId.split("?")[0];
  if (!sourcePath) {
    return code;
  }

  try {
    if (fs.statSync(sourcePath).size > MAX_UNOCSS_ORIGINAL_SOURCE_BYTES) {
      return code;
    }
  } catch {
    return code;
  }

  try {
    return `${code}\n${fs.readFileSync(sourcePath, "utf-8")}`;
  } catch {
    return code;
  }
}

export function patchUnoCssBridge(plugins: UnoCssLikePlugin[]): void {
  for (const plugin of plugins) {
    if (
      !plugin.name?.startsWith("unocss:") ||
      typeof plugin.transform !== "function" ||
      plugin[bridgePatched]
    ) {
      continue;
    }

    const originalTransform = plugin.transform;
    const isExtractionOnly = plugin.name.startsWith("unocss:global");

    plugin.transform = function (
      this: unknown,
      code: string,
      id: string,
      ...args: unknown[]
    ): unknown {
      if (!isUnoCssBridgeModuleId(id)) {
        return originalTransform.call(this, code, id, ...args);
      }

      const normalizedId = normalizeUnoCssBridgeModuleId(id);
      let effectiveCode = code;

      if (isExtractionOnly) {
        effectiveCode = appendOriginalVueSourceForUnoCss(code, normalizedId);
      }

      return originalTransform.call(this, effectiveCode, normalizedId, ...args);
    };

    plugin[bridgePatched] = true;
  }
}
