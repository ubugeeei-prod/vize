import type { ResolvedVizeConfig, VizeOptions } from "../types.ts";
import type { VizePluginState } from "./state.ts";
import { isLegacyVueVersion } from "./vue-version.ts";

export function shouldExtractCssForBuild(
  state: Pick<VizePluginState, "extractCss" | "isProduction">,
  context: { environment?: { name?: string } },
): boolean {
  if (!state.isProduction) {
    return false;
  }

  const environmentName = context.environment?.name;
  if (environmentName === "client" || environmentName === "browser") {
    return true;
  }
  if (environmentName === "ssr" || environmentName === "server") {
    return false;
  }

  return state.extractCss;
}

export function resolveCompatibilityOptions(
  options: VizeOptions,
  compilerConfig: ResolvedVizeConfig["compiler"] = {},
): NonNullable<VizeOptions["compatibility"]> {
  const compatibility = {
    ...compilerConfig.compatibility,
    ...options.compatibility,
  };
  const vueVersion = options.vueVersion ?? compatibility.vueVersion ?? 3;

  if (compatibility.hostCompiler === undefined && isLegacyVueVersion(vueVersion)) {
    compatibility.hostCompiler = true;
  }

  return compatibility;
}

export function aliasSortKey(find: string | RegExp): number {
  return typeof find === "string" ? find.length : find.source.length;
}
