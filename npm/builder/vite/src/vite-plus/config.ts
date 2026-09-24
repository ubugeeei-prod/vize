import type { ConfigEnv, UserConfig } from "vite-plus";
import type { CompilerConfig, VueVersion } from "../../../../cli/src/types/index.ts";
import type { VizeCompilerOptions } from "./types.ts";
import type { UserConfigExport } from "../types.ts";
import type {
  VizePlusOptions,
  VizeLintOptions,
  VizeSharedConfig,
  VizeTaskConfig,
  VueConfig,
  VueConfigObject,
} from "./types.ts";
import { sourceConfigKey } from "./types.ts";
import { resolveVitePlus } from "./runtime.ts";
import { loadConfig, resolveConfigExport } from "../config.ts";
import { mergeSharedConfig } from "../plugin/shared-config.ts";

export async function resolveConfig(
  config: VueConfig,
  env: ConfigEnv,
  parents = new Set<unknown>(),
): Promise<VueConfigObject> {
  if (parents.has(config)) throw new Error("Circular extends in Vize's Vite+ config");
  parents.add(config);
  try {
    if (typeof config === "function" && sourceConfigKey in config) {
      return resolveConfig(config[sourceConfigKey] as VueConfig, env, parents);
    }
    const { extends: bases, ...own } = await (typeof config === "function" ? config(env) : config);
    if (!bases) return own;
    let result: VueConfigObject = {};
    const { mergeConfig } = resolveVitePlus().require("vite-plus") as typeof import("vite-plus");
    for (const base of Array.isArray(bases) ? bases : bases ? [bases] : []) {
      result = mergeConfig(result, await resolveConfig(base, env, parents));
    }
    return mergeConfig(result, own);
  } finally {
    parents.delete(config);
  }
}

export function normalizeConfig(source: VueConfigObject, integration: VizePlusOptions) {
  const { compiler: compilerOption, typecheck, vize, lint, fmt, extends: _bases, ...rest } = source;
  const { vize: nativeLint, ...oxlint } = lint ?? {};
  const { vize: nativeFmt, ...oxfmt } = fmt ?? {};
  const alias =
    vize && typeof vize === "object" && !Array.isArray(vize)
      ? (vize as VizeSharedConfig).lint
      : undefined;
  const lintConfig =
    typeof alias === "object" && typeof nativeLint === "object"
      ? { ...alias, ...nativeLint, rules: { ...alias.rules, ...nativeLint.rules } }
      : nativeLint === undefined
        ? alias
        : nativeLint;
  const compiler = compilerOption ?? integration.plugin ?? {};
  const options: VizePlusOptions = {
    ...integration,
    check:
      typecheck === false
        ? false
        : typeof typecheck === "object"
          ? typecheck.enabled !== false
          : (typecheck ?? integration.check),
    lint: lintConfig === undefined ? integration.lint : lintConfig !== false,
    fmt: nativeFmt === undefined ? integration.fmt : nativeFmt !== false,
  };
  const config: UserConfigExport = async (env) => {
    let base = vize;
    if (vize && typeof vize === "object" && !Array.isArray(vize)) {
      const { lint: _alias, ...shared } = vize as VizeSharedConfig;
      base = shared;
    }
    const inherited =
      base === undefined
        ? await loadConfig(process.cwd(), { env })
        : await resolveConfigExport(base, env);
    const overrides = await resolveConfigExport(
      {
        ...(compiler === false ? {} : { compiler: nativeCompiler(compiler) }),
        ...(typeof typecheck === "object" ? { typeChecker: typecheck } : {}),
        ...(typeof lintConfig === "object" ? { linter: withoutTaskLintOptions(lintConfig) } : {}),
        ...(typeof nativeFmt === "object" ? { formatter: nativeFmt } : {}),
      },
      env,
    );
    return mergeSharedConfig(inherited, overrides) ?? {};
  };
  const metadata: VizeTaskConfig = {
    config,
    options,
    lintTypecheck: typeof lintConfig === "object" && lintConfig.typecheck === true,
    lintLocale: typeof lintConfig === "object" ? lintConfig.locale : undefined,
    lintHelpLevel: typeof lintConfig === "object" ? lintConfig.helpLevel : undefined,
    fmtIgnorePatterns: oxfmt.ignorePatterns,
  };
  const vp: UserConfig = { ...rest, lint: lint ? oxlint : undefined, fmt: fmt ? oxfmt : undefined };
  return { vp, metadata, compiler };
}

function withoutTaskLintOptions<T extends VizeLintOptions>(
  config: T,
): Omit<T, "typecheck" | "locale" | "helpLevel"> {
  const { typecheck: _typecheck, locale: _locale, helpLevel: _helpLevel, ...native } = config;
  return native;
}

function nativeCompiler(options: VizeCompilerOptions): CompilerConfig {
  const { compatibility, ...compiler } = options;
  const version = options.vueVersion ?? compatibility?.vueVersion;
  const versions: Record<NonNullable<typeof version>, VueVersion> = {
    0.11: "0.11",
    1: "1",
    2: "2",
    "2.7": "2.7",
    3: "3",
    legacy: "2",
  };
  return {
    ...compiler,
    compatibility: {
      ...compatibility,
      vueVersion: version === undefined ? undefined : versions[version],
    },
  };
}
