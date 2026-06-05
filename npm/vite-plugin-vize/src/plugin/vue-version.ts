import type { Plugin } from "vite";

import type { VizeOptions } from "../types.ts";
import { createLogger } from "../transform.ts";

export function isLegacyVueVersion(version: VizeOptions["vueVersion"] | undefined): boolean {
  return version === "legacy" || version === 0.11 || version === 1 || version === 2;
}

export function isLegacyVueCompatibilityMode(options: VizeOptions): boolean {
  const vueVersion = options.vueVersion ?? options.compatibility?.vueVersion;
  const hostCompiler = options.compatibility?.hostCompiler ?? isLegacyVueVersion(vueVersion);
  return hostCompiler && isLegacyVueVersion(vueVersion);
}

export function createLegacyVueCompatibilityPlugin(options: VizeOptions): Plugin {
  return {
    name: "vite-plugin-vize:legacy-vue-compat",
    configResolved(resolvedConfig) {
      createLogger(options.debug ?? false).log(
        `Legacy Vue compatibility mode is active for ${resolvedConfig.root}; Vize will not compile .vue files.`,
      );
    },
  };
}
