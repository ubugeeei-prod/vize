import fs from "node:fs";

import { VIZE_SSR_PREFIX } from "../virtual.ts";

type UnoCssLikePlugin = {
  name?: string;
  transform?: (...args: unknown[]) => unknown;
  [bridgePatched]?: boolean;
};

const bridgePatched = Symbol("vize.unocssBridgePatched");

const plainSsrPrefix = VIZE_SSR_PREFIX.slice(1);

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
        try {
          const originalSource = fs.readFileSync(normalizedId.split("?")[0]!, "utf-8");
          effectiveCode = `${code}\n${originalSource}`;
        } catch {
          // Ignore missing virtual sources and keep the compiled code path.
        }
      }

      return originalTransform.call(this, effectiveCode, normalizedId, ...args);
    };

    plugin[bridgePatched] = true;
  }
}
