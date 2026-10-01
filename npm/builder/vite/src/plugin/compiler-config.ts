import type { ResolvedVizeConfig, VizeOptions } from "../types.ts";
import { resolveExperimentalCompilerOptions } from "./experimentals.ts";
import { resolveCompatibilityOptions } from "./index-helpers.ts";

export function mergeCompilerOptions(
  options: VizeOptions,
  sharedConfig: ResolvedVizeConfig | null,
): VizeOptions {
  const viteConfig = sharedConfig?.vite ?? {};
  const compilerConfig = sharedConfig?.compiler ?? {};
  const compatibility = resolveCompatibilityOptions(options, compilerConfig);
  const vueVersion = options.vueVersion ?? compatibility.vueVersion ?? 3;
  const mode =
    options.mode ??
    compilerConfig.mode ??
    (compatibility.scriptSetupInStandalone === true ? "function" : "module");
  const templateSyntax = options.templateSyntax ?? compilerConfig.templateSyntax ?? "standard";

  return {
    ...options,
    ssr: options.ssr ?? compilerConfig.ssr ?? false,
    sourceMap: options.sourceMap ?? compilerConfig.sourceMap,
    ...resolveExperimentalCompilerOptions(options, compilerConfig, sharedConfig?.experimentals),
    customRenderer: options.customRenderer ?? compilerConfig.customRenderer ?? false,
    customElements: options.customElements ?? compilerConfig.customElements,
    templateSyntax,
    whitespace: options.whitespace ?? compilerConfig.whitespace,
    compatibility,
    vueVersion,
    mode,
    runtimeModuleName: options.runtimeModuleName ?? compilerConfig.runtimeModuleName ?? "vue",
    runtimeGlobalName: options.runtimeGlobalName ?? compilerConfig.runtimeGlobalName ?? "Vue",
    include: options.include ?? viteConfig.include,
    exclude: options.exclude ?? viteConfig.exclude,
    scanPatterns: options.scanPatterns ?? viteConfig.scanPatterns,
    precompileBatchSize: options.precompileBatchSize ?? viteConfig.precompileBatchSize,
    ignorePatterns: options.ignorePatterns ?? viteConfig.ignorePatterns,
  };
}
