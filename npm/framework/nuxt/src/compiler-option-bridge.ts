import type {
  VizeNuxtCompilerCompatibilityOptions,
  VizeNuxtCompilerOptions,
  VizeNuxtVueVersion,
} from "./compiler-options.ts";
import type { VizeNuxtCompatibilityOptions, VizeNuxtOptions } from "./options.ts";
import { buildNuxtCompilerOptions } from "./utils.ts";

export function unsupportedNuxtVueCompilerOptions(
  options: Record<string, unknown> | undefined,
): string[] {
  return Object.entries(options ?? {})
    .filter(
      ([key, value]) =>
        value !== undefined &&
        (key !== "whitespace" || (value !== "condense" && value !== "preserve")),
    )
    .map(([key]) => key);
}

function isLegacyVueVersion(version: VizeNuxtVueVersion | undefined): boolean {
  return (
    version === 0.11 || version === 1 || version === 2 || version === "2.7" || version === "legacy"
  );
}

function normalizeNuxtCompilerCompatibilityOptions(
  compatibility: VizeNuxtCompatibilityOptions,
): VizeNuxtCompilerCompatibilityOptions {
  const normalized: VizeNuxtCompilerCompatibilityOptions = {};
  const legacyHost =
    isLegacyVueVersion(compatibility.vueVersion) || compatibility.nuxtVersion === 2;

  if (compatibility.vueVersion !== undefined) {
    normalized.vueVersion = compatibility.vueVersion;
  }
  if (compatibility.hostCompiler !== undefined || legacyHost) {
    normalized.hostCompiler = compatibility.hostCompiler ?? true;
  }
  if (compatibility.scriptSetupInStandalone !== undefined) {
    normalized.scriptSetupInStandalone = compatibility.scriptSetupInStandalone;
  }
  if (compatibility.optionsApiVapor !== undefined) {
    normalized.optionsApiVapor = compatibility.optionsApiVapor;
  }
  if (compatibility.nuxtVersion !== undefined) {
    normalized.nuxtVersion = compatibility.nuxtVersion;
  }
  if (compatibility.webpackVersion !== undefined) {
    normalized.webpackVersion = compatibility.webpackVersion;
  }

  return normalized;
}

export function resolveNuxtCompilerOptions(
  rootDir: string,
  baseURL: string | undefined,
  buildAssetsDir: string | undefined,
  compiler: VizeNuxtOptions["compiler"],
  compatibility: VizeNuxtCompatibilityOptions & { supportsViteCompiler?: boolean } = {},
  vueCompilerOptions?: Record<string, unknown>,
): VizeNuxtCompilerOptions | false {
  if (compiler === false) {
    return false;
  }

  if (compatibility.supportsViteCompiler === false && compatibility.forceViteCompiler !== true) {
    return false;
  }

  const compatibilityOptions = normalizeNuxtCompilerCompatibilityOptions(compatibility);
  const unsupported = unsupportedNuxtVueCompilerOptions(vueCompilerOptions);
  if (unsupported.length > 0) {
    compatibilityOptions.hostCompiler = true;
  }
  const hasCompatibilityOptions = Object.keys(compatibilityOptions).length > 0;
  const overrides = typeof compiler === "object" && compiler != null ? compiler : {};
  return buildNuxtCompilerOptions(rootDir, baseURL, buildAssetsDir, {
    vueVersion: compatibility.vueVersion,
    ...(hasCompatibilityOptions ? { compatibility: compatibilityOptions } : {}),
    mode: compatibility.scriptSetupInStandalone === true ? "function" : undefined,
    ...(vueCompilerOptions?.whitespace === "condense" ||
    vueCompilerOptions?.whitespace === "preserve"
      ? { whitespace: vueCompilerOptions.whitespace }
      : {}),
    ...overrides,
    ...(unsupported.length > 0 ? { compatibility: compatibilityOptions } : {}),
  });
}
