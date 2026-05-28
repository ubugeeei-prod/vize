import type { Plugin } from "vite";

import type { VizeOptions } from "../types.ts";
import { createLogger } from "../transform.ts";

export function isLegacyVueCompatibilityMode(options: VizeOptions): boolean {
  return options.vueVersion !== undefined && options.vueVersion !== 3;
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
