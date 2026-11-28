import type { VizeNuxtCompilerOptions } from "./compiler-options.ts";
import type { VizeNuxtOptions } from "./options.ts";
import { splitNuxtCompilerDefaults } from "./compiler-option-bridge.ts";
import { placeVizeCompileStage } from "./host-vue-bridge.ts";
import { setupNuxtTemplateDiagnostics } from "./template-diagnostics.ts";
import { resolve } from "node:path";

type NuxtCompilerHost = {
  options: {
    srcDir?: string;
    buildDir?: string;
    typesDir?: string;
    vue?: { compilerOptions?: Record<string, unknown> };
    vite?: { plugins?: unknown[] };
    experimental?: { templateDiagnostics?: boolean };
  };
  hook(name: string, callback: (...args: unknown[]) => unknown): void;
};

/**
 * Nuxt writes its generated root tsconfig to `typesDir` (Nuxt 4.7+), falling
 * back to `buildDir` before that option existed. A production build that runs
 * after `nuxt prepare` or `nuxt dev` builds into a cache `buildDir` and keeps
 * the tsconfigs in `typesDir`, so `buildDir` is not where they are.
 */
export function nuxtTypeResolutionFallback(
  host: Pick<NuxtCompilerHost["options"], "buildDir" | "typesDir">,
): { typeResolutionFallbackTsconfig?: string } {
  const typesDir = host.typesDir || host.buildDir;
  return typesDir ? { typeResolutionFallbackTsconfig: resolve(typesDir, "tsconfig.json") } : {};
}

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
    {
      ...explicit,
      nuxtPageMeta: true,
      ssrModuleIdRoot: host.srcDir,
      // Native selection still prefers a governing user config for each source.
      ...nuxtTypeResolutionFallback(host),
    },
    defaults,
    "nuxt",
  );
  host.vite.plugins.push(plugins);
  // The host compiler's node transforms report Nuxt's template diagnostics.
  setupNuxtTemplateDiagnostics(nuxt, host.vite.plugins);
  const stage = plugins.find((plugin) => plugin.name === "vite-plugin-vize:sfc");
  if (!stage) return;
  nuxt.hook("vite:configResolved", (config) => {
    placeVizeCompileStage((config as { plugins: unknown[] }).plugins, stage);
  });
}
