import type { ConfigEnv, UserConfig } from "vite-plus";
import type { UserConfigInput, VueVersion } from "../../../../cli/src/types/index.ts";
import type { VizeCompilerOptions } from "./types.ts";
import type { UserConfigExport } from "../types.ts";
import type { VizePlusOptions, VizeTaskConfig, VueConfig, VueConfigObject } from "./types.ts";
import { sourceConfigKey } from "./types.ts";
import {
  projectViteConfigSource,
  vueConfigOptions,
} from "../../../../cli/src/config/vite-runtime.mjs";

export async function resolveConfig(config: VueConfig, env: ConfigEnv): Promise<VueConfigObject> {
  if (
    config !== null &&
    (typeof config === "function" || typeof config === "object") &&
    sourceConfigKey in config
  ) {
    return resolveConfig(config[sourceConfigKey] as VueConfig, env);
  }
  const source = await (typeof config === "function" ? config(env) : config);
  if ("extends" in source) {
    throw new Error(
      "Custom config extends is no longer supported; use defineConfig(vpConfig).withVue(vueConfig)",
    );
  }
  assertVueConfigSpelling(source);
  return source;
}

function assertVueConfigSpelling(source: VueConfigObject) {
  const legacy =
    "vize" in source
      ? "vize"
      : source.lint && "vize" in source.lint
        ? "lint.vize"
        : source.fmt && "vize" in source.fmt
          ? "fmt.vize"
          : undefined;
  if (legacy) {
    if (legacy === "lint.vize") {
      throw new Error(
        "lint.vize is no longer supported; pass { lint: false } as defineConfig's integration options to disable the Vite+ native lint task while keeping CLI lint enabled, and put shared lint settings in vue.linter",
      );
    }
    throw new Error(
      `${legacy}: configuration is no longer supported; move Vue settings to vue: in vite.config.*`,
    );
  }
}

export function normalizeConfig(source: VueConfigObject, integration: VizePlusOptions) {
  assertVueConfigSpelling(source);
  const { compiler: compilerOption, typecheck, lint, fmt, vue: _vue, ...rest } = source;
  const vue = vueConfigOptions(source);
  const hostLint = vue?.lint;
  const hostVizeSettings = hostLint?.settings.vize;
  const configDirectory = process.cwd();
  const { vue: nativeLint, ...oxlint } = lint ?? {};
  const { vue: nativeFmt, ...oxfmt } = fmt ?? {};
  const lintConfig = nativeLint;
  const compiler = vueCompiler(compilerOption ?? integration.plugin ?? {}, vue);
  const options: VizePlusOptions = {
    ...integration,
    check:
      (vue?.typeCheck ?? vue?.typeChecker)?.enabled !== undefined
        ? (vue?.typeCheck ?? vue?.typeChecker)?.enabled
        : typecheck === false
          ? false
          : typeof typecheck === "object"
            ? typecheck.enabled !== false
            : (typecheck ?? integration.check),
    lint:
      hostLint !== undefined
        ? false
        : vue?.linter?.enabled !== undefined
          ? vue.linter.enabled
          : lintConfig === undefined
            ? integration.lint
            : lintConfig !== false,
    fmt:
      (vue?.fmt ?? vue?.formatter) !== undefined
        ? true
        : nativeFmt === undefined
          ? integration.fmt
          : nativeFmt !== false,
  };
  const config: UserConfigExport = async (env) =>
    (await projectViteConfigSource(source, env, configDirectory)) as UserConfigInput;
  const metadata: VizeTaskConfig = {
    config,
    options,
    lintTypecheck: typeof lintConfig === "object" && lintConfig.typecheck === true,
    lintLocale:
      hostVizeSettings?.locale === "en" ||
      hostVizeSettings?.locale === "ja" ||
      hostVizeSettings?.locale === "zh"
        ? hostVizeSettings.locale
        : typeof lintConfig === "object"
          ? lintConfig.locale
          : undefined,
    lintHelpLevel:
      hostVizeSettings?.helpLevel === "full" ||
      hostVizeSettings?.helpLevel === "short" ||
      hostVizeSettings?.helpLevel === "none"
        ? hostVizeSettings.helpLevel
        : typeof lintConfig === "object"
          ? lintConfig.helpLevel
          : undefined,
    fmtIgnorePatterns: oxfmt.ignorePatterns,
  };
  const vp: UserConfig = {
    ...rest,
    // Both Vite and Vite+ accept named plugins, but their hook types may be
    // instantiated from different Vite packages in a consumer's dependency tree.
    plugins: rest.plugins as UserConfig["plugins"],
    lint: (hostLint ?? (lint ? oxlint : undefined)) as UserConfig["lint"],
    fmt: fmt ? oxfmt : undefined,
  };
  return { vp, metadata, compiler };
}

/** Keep plugin-only fields while native canonical leaves choose their own values. */
function vueCompiler(
  legacy: VizeCompilerOptions | false,
  vue: VueConfigObject["vue"],
): VizeCompilerOptions | false {
  if (legacy === false) return false;
  const compiler = vue?.compile ?? vue?.compiler;
  if (compiler === undefined && vue?.version === undefined) return legacy;
  const { compatibility, ...native } = compiler ?? {};
  const { vueVersion: _nativeVersion, ...nativeCompatibility } = compatibility ?? {};
  const version = vue?.version ?? compatibility?.vueVersion;
  const versions: Record<VueVersion, NonNullable<VizeCompilerOptions["vueVersion"]>> = {
    "0.10": 0.1,
    "0.11": 0.11,
    "1": 1,
    "2": 2,
    "2.7": "2.7",
    "3": 3,
  };
  const previous = legacy;
  return {
    ...previous,
    ...native,
    compatibility: {
      ...previous.compatibility,
      ...nativeCompatibility,
      ...(version === undefined ? {} : { vueVersion: versions[version] }),
    },
    ...(version === undefined ? {} : { vueVersion: versions[version] }),
  };
}
