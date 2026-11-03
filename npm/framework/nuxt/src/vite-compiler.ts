import type { VizeNuxtCompilerOptions } from "./compiler-options.ts";
import type { VizeNuxtOptions } from "./options.ts";
import { splitNuxtCompilerDefaults } from "./compiler-option-bridge.ts";

type NuxtCompilerHost = {
  srcDir?: string;
  vue?: { compilerOptions?: Record<string, unknown> };
  vite?: { plugins?: unknown[] };
};

/**
 * Keep lazy plugin loading and native-only registration outside the module
 * entry. Returns Vize's SFC compile stage, which takes the host compiler's place.
 */
export async function setupNuxtViteCompiler(
  compilerOptions: VizeNuxtCompilerOptions,
  host: NuxtCompilerHost,
  compiler: VizeNuxtOptions["compiler"],
): Promise<object | undefined> {
  const { default: vize } = await import("@vizejs/vite-plugin");
  const { options: explicit, defaults } = splitNuxtCompilerDefaults(
    compilerOptions,
    compiler,
    host.vue?.compilerOptions,
  );
  host.vite ||= {};
  host.vite.plugins = host.vite.plugins || [];
  const plugins = vize(
    { ...explicit, nuxtPageMeta: true, ssrModuleIdRoot: host.srcDir },
    defaults,
    "nuxt",
  );
  host.vite.plugins.push(plugins);
  return plugins.find((plugin) => plugin.name === "vite-plugin-vize:sfc");
}
