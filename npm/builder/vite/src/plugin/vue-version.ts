import type { Plugin } from "vite";

import type { VizeOptions } from "../types.ts";
import { createLogger } from "../transform.ts";

export function isLegacyVueVersion(version: VizeOptions["vueVersion"] | undefined): boolean {
  return (
    version === "legacy" ||
    version === 0.1 ||
    version === 0.11 ||
    version === 1 ||
    version === 2 ||
    version === "2.7"
  );
}

export function isLegacyVueCompatibilityMode(options: VizeOptions): boolean {
  const vueVersion = options.vueVersion ?? options.compatibility?.vueVersion;
  const hostCompiler = options.compatibility?.hostCompiler ?? isLegacyVueVersion(vueVersion);
  return hostCompiler && isLegacyVueVersion(vueVersion);
}

export function hasHostVueSfcCompilerPlugin(plugins: readonly Pick<Plugin, "name">[]): boolean {
  return plugins.some((plugin) => {
    const name = plugin.name;
    return (
      name === "vite:vue" ||
      name === "vite:vue2" ||
      name.includes("plugin-vue") ||
      name.includes("vue2")
    );
  });
}

export function createLegacyVueCompatibilityPlugin(options: VizeOptions): Plugin {
  return {
    name: "vite-plugin-vize:legacy-vue-compat",
    configResolved(resolvedConfig) {
      if (!hasHostVueSfcCompilerPlugin(resolvedConfig.plugins)) {
        throw new Error(
          "vite-plugin-vize: legacy Vue host-compiler mode requires a host Vue SFC compiler plugin. Add @vitejs/plugin-vue2 (or your framework's Vue 2 Vite SFC plugin), or set compatibility.hostCompiler=false if this project can use Vize's compiler pipeline.",
        );
      }

      createLogger(options.debug ?? false).log(
        `Legacy Vue compatibility mode is active for ${resolvedConfig.root}; Vize will not compile .vue files.`,
      );
    },
  };
}

type HostVuePlugin = Pick<Plugin, "name" | "load" | "transform">;

/**
 * Remove `@vitejs/plugin-vue` instances that sit beside Vize, such as the one a
 * framework adapter bundles (`@void/vue`'s `voidVue()`), so one compiler owns
 * each `.vue` module. Vize's own `vite:vue` shim has no compiler hooks and stays.
 * An explicit `exclude` keeps the host compiler for the SFCs Vize skips.
 */
export function removeHostVueSfcCompilers(
  plugins: HostVuePlugin[],
  options: Pick<VizeOptions, "exclude">,
): number {
  if (options.exclude !== undefined) return 0;
  let removed = 0;
  for (let index = plugins.length - 1; index >= 0; index--) {
    const plugin = plugins[index];
    if (plugin.name === "vite:vue" && (plugin.transform || plugin.load)) {
      plugins.splice(index, 1);
      removed++;
    }
  }
  return removed;
}
