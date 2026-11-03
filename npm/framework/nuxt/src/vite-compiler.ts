import type { VizeNuxtCompilerOptions } from "./compiler-options.ts";
import type { VizeNuxtOptions } from "./options.ts";
import { splitNuxtCompilerDefaults } from "./compiler-option-bridge.ts";
import { placeVizeCompileStage } from "./host-vue-bridge.ts";

type NuxtCompilerHost = {
  options: {
    srcDir?: string;
    vue?: { compilerOptions?: Record<string, unknown> };
    vite?: { plugins?: unknown[] };
  };
  hook(name: string, callback: (...args: unknown[]) => unknown): void;
};

/**
 * Keep lazy plugin loading and native-only registration outside the module
 * entry. Vize's SFC compile stage takes the host compiler's place in each
 * config; this hook runs before the module's own, which drops that compiler.
 */
export async function setupNuxtViteCompiler(
  compilerOptions: VizeNuxtCompilerOptions,
  nuxt: NuxtCompilerHost,
  compiler: VizeNuxtOptions["compiler"],
): Promise<void> {
  const { default: vize } = await import("@vizejs/vite-plugin");
  const host = nuxt.options;
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
  const stage = plugins.find((plugin) => plugin.name === "vite-plugin-vize:sfc");
  if (!stage) return;
  nuxt.hook("vite:configResolved", (config) => {
    placeVizeCompileStage((config as { plugins: unknown[] }).plugins, stage);
  });
}
