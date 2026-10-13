import type { ConfigEnv, UserConfig } from "vite-plus";
import type {
  CompilerConfig,
  UserConfigInput,
  VueVersion,
} from "../../../../cli/src/types/index.ts";
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
import {
  mergeVueConfigOptions,
  projectViteConfigSource,
  vueConfigOptions,
} from "../../../../cli/src/config/vite-runtime.mjs";

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
    const compose = (base: VueConfigObject, selected: VueConfigObject): VueConfigObject => {
      const merged = mergeConfig(base, selected);
      const vue = mergeVueConfigOptions(base.vue, selected.vue);
      return vue === undefined ? merged : { ...merged, vue };
    };
    for (const base of Array.isArray(bases) ? bases : bases ? [bases] : []) {
      result = compose(result, await resolveConfig(base, env, parents));
    }
    return compose(result, own);
  } finally {
    parents.delete(config);
  }
}

export function normalizeConfig(source: VueConfigObject, integration: VizePlusOptions) {
  const {
    compiler: compilerOption,
    typecheck,
    vize,
    lint,
    fmt,
    vue: _vue,
    extends: _bases,
    ...rest
  } = source;
  const vue = vueConfigOptions(source);
  const configDirectory = process.cwd();
  const { vize: nativeLint, ...oxlint } = lint ?? {};
  const { vize: nativeFmt, ...oxfmt } = fmt ?? {};
  const formatter = {
    ...(oxfmt.sortImports === undefined ? {} : { sortImports: oxfmt.sortImports }),
    ...(typeof nativeFmt === "object" ? nativeFmt : {}),
  };
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
  const compiler = vueCompiler(compilerOption ?? integration.plugin ?? {}, vue);
  const options: VizePlusOptions = {
    ...integration,
    check:
      vue?.typeChecker?.enabled !== undefined
        ? vue.typeChecker.enabled
        : typecheck === false
          ? false
          : typeof typecheck === "object"
            ? typecheck.enabled !== false
            : (typecheck ?? integration.check),
    lint:
      vue?.linter?.enabled !== undefined
        ? vue.linter.enabled
        : lintConfig === undefined
          ? integration.lint
          : lintConfig !== false,
    fmt:
      vue?.formatter !== undefined
        ? true
        : nativeFmt === undefined
          ? integration.fmt
          : nativeFmt !== false,
  };
  const legacyConfig: UserConfigExport = async (env) => {
    let base = vize;
    if (vize && typeof vize === "object" && !Array.isArray(vize)) {
      const { lint: _alias, ...shared } = vize as VizeSharedConfig;
      base = shared;
    }
    const inherited =
      base === undefined
        ? await loadConfig(process.cwd(), { env, viteConfig: false })
        : await resolveConfigExport(base, env);
    const overrides = await resolveConfigExport(
      {
        ...(compiler === false ? {} : { compiler: nativeCompiler(compiler) }),
        ...(typeof typecheck === "object" ? { typeChecker: typecheck } : {}),
        ...(typeof lintConfig === "object" ? { linter: withoutTaskLintOptions(lintConfig) } : {}),
        ...(Object.keys(formatter).length ? { formatter } : {}),
      },
      env,
    );
    // Fresh native checks support JSX without changing formatter task payloads
    // or the historical defaults inherited from a dedicated config file.
    const defaults =
      env.command === "check" && (base !== undefined || inherited === null)
        ? await resolveConfigExport({ typeChecker: { jsxTypecheck: true } }, env)
        : null;
    return mergeSharedConfig(mergeSharedConfig(defaults, inherited), overrides) ?? {};
  };
  const config: UserConfigExport =
    vue === undefined
      ? legacyConfig
      : async (env) =>
          (await projectViteConfigSource(source, env, configDirectory)) as UserConfigInput;
  const metadata: VizeTaskConfig = {
    config,
    options,
    lintTypecheck: typeof lintConfig === "object" && lintConfig.typecheck === true,
    lintLocale: typeof lintConfig === "object" ? lintConfig.locale : undefined,
    lintHelpLevel: typeof lintConfig === "object" ? lintConfig.helpLevel : undefined,
    fmtIgnorePatterns: oxfmt.ignorePatterns,
  };
  const vp: UserConfig = {
    ...rest,
    // Both Vite and Vite+ accept named plugins, but their hook types may be
    // instantiated from different Vite packages in a consumer's dependency tree.
    plugins: rest.plugins as UserConfig["plugins"],
    lint: lint ? oxlint : undefined,
    fmt: fmt ? oxfmt : undefined,
  };
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
    0.1: "0.10",
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

/** Keep plugin-only fields while native canonical leaves choose their own values. */
function vueCompiler(
  legacy: VizeCompilerOptions | false,
  vue: VueConfigObject["vue"],
): VizeCompilerOptions | false {
  if (vue?.compiler === undefined && vue?.version === undefined) return legacy;
  if (legacy === false && vue.compiler === undefined) return false;
  const { compatibility, ...native } = vue.compiler ?? {};
  const version = vue.version ?? compatibility?.vueVersion;
  const versions: Record<VueVersion, NonNullable<VizeCompilerOptions["vueVersion"]>> = {
    "0.10": 0.1,
    "0.11": 0.11,
    "1": 1,
    "2": 2,
    "2.7": "2.7",
    "3": 3,
  };
  const previous = legacy === false ? {} : legacy;
  return {
    ...previous,
    ...native,
    compatibility: {
      ...previous.compatibility,
      ...compatibility,
      ...(version === undefined ? {} : { vueVersion: versions[version] }),
    },
    ...(version === undefined ? {} : { vueVersion: versions[version] }),
  };
}
