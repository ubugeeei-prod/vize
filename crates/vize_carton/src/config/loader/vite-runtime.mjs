// This dependency-free projection is also embedded by the native config loader.
import { resolve } from "node:path";

// Read the source of our Vite+ factory without starting Vite, plugins or tasks.
const sourceConfigKey = Symbol.for("@vizejs/vite-plugin/vite-plus/source");

function object(value) {
  if (value === null || typeof value !== "object") return false;
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function merge(base, own) {
  const result = { ...base };
  for (const [key, value] of Object.entries(own)) {
    result[key] = object(value) && object(result[key]) ? merge(result[key], value) : value;
  }
  return result;
}

async function resolveSource(value, env, parents = new Set()) {
  value = await value;
  if (parents.has(value)) throw new Error("Circular extends in Vize's Vite config");
  parents.add(value);
  try {
    if (typeof value === "function" && sourceConfigKey in value) {
      return await resolveSource(value[sourceConfigKey], env, parents);
    }
    const config = await (typeof value === "function" ? value(env) : value);
    if (!object(config)) throw new Error("Vite config must export an object or a config function");
    const { extends: bases, ...own } = config;
    let inherited = {};
    for (const base of Array.isArray(bases) ? bases : bases ? [bases] : []) {
      inherited = merge(inherited, await resolveSource(base, env, parents));
    }
    return merge(inherited, own);
  } finally {
    parents.delete(value);
  }
}

export const VITE_CONFIG_FILE_NAMES = [
  "vite.config.ts",
  "vite.config.js",
  "vite.config.mjs",
  "vite.config.cjs",
  "vite.config.mts",
  "vite.config.cts",
];

export function isViteConfigFile(filePath) {
  return /(?:^|[/\\])vite\.config\.(?:[cm]?[jt]s)$/.test(filePath);
}

/** Project workflows support JSX without an extra feature flag. Legacy config stays unchanged. */
export function projectConfigDefaults() {
  return { typeChecker: { jsxTypecheck: true } };
}

/** Resolve just the native settings from an ordinary Vite or Vite+ export. */
export async function resolveViteConfigExport(exported, env, configDir) {
  env ??= { mode: "development", command: "serve" };
  const source = await resolveSource(exported, env);
  let shared = await (typeof source.vize === "function" ? source.vize(env) : source.vize);
  const alias = object(shared) ? shared.lint : undefined;
  if (object(shared)) {
    const { lint: _lint, ...native } = shared;
    shared = native;
  }
  const overrides = {};
  if (object(source.compiler)) {
    const { vueVersion, ...compiler } = source.compiler;
    const version = vueVersion ?? compiler.compatibility?.vueVersion;
    overrides.compiler = merge(
      compiler,
      version === undefined
        ? {}
        : {
            compatibility: { vueVersion: String(version === "legacy" ? 2 : version) },
          },
    );
  }
  if (typeof source.typecheck === "boolean") overrides.typeChecker = { enabled: source.typecheck };
  else if (object(source.typecheck)) overrides.typeChecker = source.typecheck;
  const lint = source.lint?.vize === undefined ? alias : source.lint.vize;
  if (typeof lint === "boolean") overrides.linter = { enabled: lint };
  else if (object(lint)) {
    const {
      typecheck: _typecheck,
      locale: _locale,
      helpLevel: _helpLevel,
      ...native
    } = merge(object(alias) ? alias : {}, lint);
    overrides.linter = native;
  }
  if (object(source.fmt)) {
    const { vize, sortImports } = source.fmt;
    const common = sortImports === undefined ? {} : { sortImports };
    overrides.formatter = merge(common, object(vize) ? vize : {});
  }
  // Keep scoped entries in their authored order; the unscoped defaults precede
  // them, while tool sections override only the shared unscoped settings.
  const root = typeof source.root === "string" ? resolve(configDir ?? ".", source.root) : configDir;
  if (Array.isArray(shared)) {
    return [
      { ...projectConfigDefaults(), ...(root ? { __vizeProjectRoot: root } : {}) },
      ...shared.map((entry) => projectPaths(entry, root, true)),
      projectPaths(overrides, root, false),
    ];
  }
  const config = merge(merge(projectConfigDefaults(), shared ?? {}), overrides);
  return root ? { ...projectPaths(config, root, false), __vizeProjectRoot: root } : config;
}

// Vite-owned settings use its root. Dedicated Vize files keep their established
// config-directory path contract, and CLI arguments never pass through here.
function projectPaths(config, root, entry) {
  if (!root || !object(config)) return config;
  const result = { ...config };
  if (
    typeof config.basePath === "string" ||
    config.files !== undefined ||
    (entry && config.ignores !== undefined)
  ) {
    result.basePath = resolve(root, config.basePath ?? ".");
  }
  if (!entry && Array.isArray(config.ignores)) {
    result.ignores = config.ignores.map((pattern) => resolve(root, pattern));
  }
  if (Array.isArray(config.entries)) {
    result.entries = config.entries.map((value) => projectPaths(value, root, true));
  }
  if (object(config.typeChecker)) {
    result.typeChecker = { ...config.typeChecker };
    for (const key of ["tsconfig", "corsaPath", "tsgoPath", "globalsFile"]) {
      if (typeof config.typeChecker[key] === "string") {
        result.typeChecker[key] = resolve(root, config.typeChecker[key]);
      }
    }
  }
  return result;
}
