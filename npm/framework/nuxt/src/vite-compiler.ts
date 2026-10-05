import type { VizeNuxtCompilerOptions } from "./compiler-options.ts";

type NuxtCompilerHost = {
  srcDir?: string;
  vite?: { plugins?: unknown[] };
};

/** Keep lazy plugin loading and native-only registration outside the module entry. */
export async function setupNuxtViteCompiler(
  compilerOptions: VizeNuxtCompilerOptions,
  host: NuxtCompilerHost,
): Promise<void> {
  const { default: vize } = await import("@vizejs/vite-plugin");
  host.vite ||= {};
  host.vite.plugins = host.vite.plugins || [];
  host.vite.plugins.push(
    vize({ ...compilerOptions, nuxtPageMeta: true, ssrModuleIdRoot: host.srcDir }),
  );
}
