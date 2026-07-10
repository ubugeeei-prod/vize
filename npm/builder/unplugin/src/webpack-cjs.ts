import type { Compiler as WebpackCompiler } from "webpack";
import type { VizeUnpluginOptions } from "./types.ts";

type WebpackPlugin = {
  apply(compiler: WebpackCompiler): void;
};

function isLegacyVueVersion(version: unknown): boolean {
  return (
    version === "legacy" ||
    version === "2" ||
    version === "2.6" ||
    version === "2.7" ||
    version === 0.11 ||
    version === 1 ||
    version === 2
  );
}

function shouldUseHostCompiler(options: VizeUnpluginOptions | undefined): boolean {
  const compatibility = options?.compatibility;
  return (
    compatibility?.hostCompiler ??
    (compatibility?.nuxtVersion === 2 ||
      isLegacyVueVersion(options?.vueVersion ?? compatibility?.vueVersion))
  );
}

function createUnsupportedCjsPlugin(): WebpackPlugin {
  return {
    apply() {
      throw new Error(
        "[vize] @vizejs/unplugin/webpack was loaded through CommonJS, which is only supported for Nuxt 2/Vue 2 host-compiler configs. Use an ESM webpack config, or pass { vueVersion: 2 } / { compatibility: { hostCompiler: true } }.",
      );
    },
  };
}

function createHostCompilerPlugin(): WebpackPlugin {
  return {
    apply() {
      // Nuxt 2/Vue 2 keeps the host Vue compiler in charge. The CJS entry exists
      // so jiti can load nuxt.config.ts without evaluating unplugin's ESM runtime.
    },
  };
}

function vizeWebpackCjs(options?: VizeUnpluginOptions): WebpackPlugin {
  return shouldUseHostCompiler(options) ? createHostCompilerPlugin() : createUnsupportedCjsPlugin();
}

export { vizeWebpackCjs };
export default vizeWebpackCjs;
