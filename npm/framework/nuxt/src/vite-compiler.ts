import type { VizeNuxtCompilerOptions } from "./compiler-options.ts";
import type { VizeNuxtOptions } from "./options.ts";
import { splitNuxtCompilerDefaults } from "./compiler-option-bridge.ts";

type NuxtCompilerHost = {
  srcDir?: string;
  vue?: { compilerOptions?: Record<string, unknown> };
  vite?: { plugins?: unknown[] };
};

/** Keep lazy plugin loading and native-only registration outside the module entry. */
export async function setupNuxtViteCompiler(
  compilerOptions: VizeNuxtCompilerOptions,
  host: NuxtCompilerHost,
  compiler: VizeNuxtOptions["compiler"],
): Promise<void> {
  const { default: vize } = await import("@vizejs/vite-plugin");
  const { options: explicit, defaults } = splitNuxtCompilerDefaults(
    compilerOptions,
    compiler,
    host.vue?.compilerOptions,
  );
  host.vite ||= {};
  host.vite.plugins = host.vite.plugins || [];
  host.vite.plugins.push(
    vize({ ...explicit, nuxtPageMeta: true, ssrModuleIdRoot: host.srcDir }, defaults),
  );
}
