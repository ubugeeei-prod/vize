import * as fs from "node:fs";
import * as path from "node:path";
import { execFileSync } from "node:child_process";
import { randomUUID } from "node:crypto";
import { fileURLToPath, pathToFileURL } from "node:url";
import { transform } from "oxc-transform";
import type {
  LanguageServerConfig,
  VizeConfig,
  VizeConfigEntry,
  ResolvedVizeConfig,
  LoadConfigOptions,
  UserConfigExport,
  ConfigEnv,
  GlobalTypesConfig,
  GlobalTypeDeclaration,
} from "./types/index.js";

export const CONFIG_FILE_NAMES = [
  "vize.config.pkl",
  "vize.config.ts",
  "vize.config.js",
  "vize.config.mjs",
  "vize.config.json",
] as const;

const DEFAULT_CONFIG_ENV: ConfigEnv = {
  mode: "development",
  command: "serve",
};

const PACKAGE_ROOT = path.resolve(fileURLToPath(new URL(".", import.meta.url)), "..");

export const VIZE_CONFIG_JSON_SCHEMA_PATH = path.join(
  PACKAGE_ROOT,
  "schemas",
  "vize.config.schema.json",
);

export const VIZE_CONFIG_PKL_SCHEMA_PATH = path.join(PACKAGE_ROOT, "pkl", "vize.pkl");

const DOCUMENTED_PKL_SCHEMA_IMPORT_RE =
  /^(\s*(?:amends|import)\s+)(["'])node_modules\/vize\/pkl\/(VizeConfig\.pkl|vize\.pkl)\2/gm;

type CompatVizeConfig = VizeConfig & {
  lsp?: LanguageServerConfig;
};

type CompatVizeConfigEntry = VizeConfigEntry & {
  lsp?: LanguageServerConfig;
};

/**
 * Define a Vize configuration with type checking.
 * Accepts a plain object or a function that receives ConfigEnv.
 */
export function defineConfig(config: UserConfigExport): UserConfigExport {
  return config;
}

/**
 * Load `vize.config.*` from the specified directory.
 */
export async function loadConfig(
  root: string,
  options: LoadConfigOptions = {},
): Promise<ResolvedVizeConfig | null> {
  const { mode = "root", configFile, env } = options;

  if (mode === "none") {
    return null;
  }

  if (configFile) {
    const absolutePath = path.isAbsolute(configFile) ? configFile : path.resolve(root, configFile);
    if (fs.existsSync(absolutePath)) {
      return loadConfigFile(absolutePath, env);
    }
    return null;
  }

  if (mode === "auto") {
    return loadConfigFromDirAuto(root, env);
  }

  return loadConfigFromDir(root, env);
}

async function loadConfigFromDir(dir: string, env?: ConfigEnv): Promise<ResolvedVizeConfig | null> {
  for (const name of CONFIG_FILE_NAMES) {
    const filePath = path.join(dir, name);
    if (!fs.existsSync(filePath)) {
      continue;
    }

    const config = await loadConfigFile(filePath, env);
    if (config !== null) {
      return config;
    }
  }
  return null;
}

async function loadConfigFromDirAuto(
  startDir: string,
  env?: ConfigEnv,
): Promise<ResolvedVizeConfig | null> {
  let currentDir = path.resolve(startDir);

  while (true) {
    const config = await loadConfigFromDir(currentDir, env);
    if (config !== null) {
      return config;
    }

    const parentDir = path.dirname(currentDir);
    if (parentDir === currentDir) {
      return null;
    }

    currentDir = parentDir;
  }
}

async function loadConfigFile(
  filePath: string,
  env?: ConfigEnv,
): Promise<ResolvedVizeConfig | null> {
  const absolutePath = path.resolve(filePath);
  if (!fs.existsSync(absolutePath)) {
    return null;
  }

  const ext = path.extname(absolutePath);

  if (ext === ".pkl") {
    return loadPklConfig(absolutePath);
  }

  if (ext === ".json") {
    const content = fs.readFileSync(absolutePath, "utf-8");
    return parseJsonConfig(content, absolutePath);
  }

  if (ext === ".ts") {
    return loadTypeScriptConfig(absolutePath, env);
  }

  return loadESMConfig(absolutePath, env);
}

function findPklBinary(): string | null {
  try {
    const pklPkgPath = import.meta.resolve?.("@pkl-community/pkl");
    if (pklPkgPath) {
      const pklLibDir = path.dirname(fileURLToPath(pklPkgPath));
      const pklPackageDir = path.dirname(pklLibDir);
      const candidates = [
        path.join(pklLibDir, "main.js"),
        path.join(pklPackageDir, "pkl"),
        path.join(pklPackageDir, "pkl.exe"),
      ];

      for (const candidate of candidates) {
        if (fs.existsSync(candidate)) {
          try {
            execFileSync(candidate, ["--version"], { stdio: "ignore" });
            return candidate;
          } catch {
            // Keep looking: the bundled shim can exist even when its runtime is unavailable.
          }
        }
      }
    }
  } catch {
    // Fall back to PATH below.
  }

  try {
    execFileSync("pkl", ["--version"], { stdio: "ignore" });
    return "pkl";
  } catch {
    return null;
  }
}

function loadPklConfig(filePath: string): ResolvedVizeConfig | null {
  const pklBin = findPklBinary();
  if (!pklBin) {
    console.warn(
      "[vize] pkl CLI not found. Install @pkl-community/pkl or add pkl to PATH. " +
        "Falling back to the next config format.",
    );
    return null;
  }

  let output: string;
  const patchedFilePath = createPklConfigWithBundledSchemaImports(filePath);
  const evalFilePath = patchedFilePath ?? filePath;
  try {
    output = execFileSync(pklBin, ["eval", "-f", "json", evalFilePath], {
      cwd: path.dirname(filePath),
      encoding: "utf-8",
      stdio: ["ignore", "pipe", "pipe"],
      timeout: 30_000,
    });
  } catch (error) {
    throw new Error(`Failed to evaluate vize PKL config at ${filePath}: ${getErrorMessage(error)}`);
  } finally {
    if (patchedFilePath) {
      fs.rmSync(patchedFilePath, { force: true });
    }
  }
  return parseJsonConfig(output, filePath);
}

function createPklConfigWithBundledSchemaImports(filePath: string): string | null {
  const configDir = path.dirname(filePath);
  const source = fs.readFileSync(filePath, "utf-8");
  let patched = false;

  const content = source.replace(
    DOCUMENTED_PKL_SCHEMA_IMPORT_RE,
    (match, prefix, quote, schemaFile) => {
      const projectSchemaPath = path.join(configDir, "node_modules", "vize", "pkl", schemaFile);
      if (fs.existsSync(projectSchemaPath)) {
        return match;
      }

      const bundledSchemaPath = path.join(PACKAGE_ROOT, "pkl", schemaFile);
      if (!fs.existsSync(bundledSchemaPath)) {
        return match;
      }

      patched = true;
      return `${prefix}${quote}${pathToFileURL(bundledSchemaPath).href}${quote}`;
    },
  );

  if (!patched) {
    return null;
  }

  const tempFile = path.join(
    configDir,
    `.vize-config-${process.pid}-${Date.now()}-${randomUUID()}.pkl`,
  );
  fs.writeFileSync(tempFile, content, { flag: "wx", mode: 0o600 });
  return tempFile;
}

export async function resolveConfigExport(
  exported: UserConfigExport,
  env?: ConfigEnv,
): Promise<ResolvedVizeConfig> {
  if (typeof exported === "function") {
    return normalizeLoadedConfig(await exported(env ?? DEFAULT_CONFIG_ENV));
  }

  return normalizeLoadedConfig(exported);
}

async function loadTypeScriptConfig(
  filePath: string,
  env?: ConfigEnv,
): Promise<ResolvedVizeConfig> {
  const source = fs.readFileSync(filePath, "utf-8");
  const result = await transform(filePath, source, {
    typescript: {
      onlyRemoveTypeImports: true,
    },
  });

  const tempFile = path.join(
    path.dirname(filePath),
    `.vize-config-${process.pid}-${Date.now()}-${randomUUID()}.mjs`,
  );
  fs.writeFileSync(tempFile, result.code, { flag: "wx", mode: 0o600 });

  try {
    const module = await importFresh(tempFile);
    const exported: UserConfigExport = module.default || module;
    return resolveConfigExport(exported, env);
  } finally {
    fs.rmSync(tempFile, { force: true });
  }
}

async function loadESMConfig(filePath: string, env?: ConfigEnv): Promise<ResolvedVizeConfig> {
  const module = await importFresh(filePath);
  const exported: UserConfigExport = module.default || module;
  return resolveConfigExport(exported, env);
}

async function importFresh(filePath: string): Promise<Record<string, unknown>> {
  const fileUrl = pathToFileURL(filePath);
  fileUrl.searchParams.set("t", String(fs.statSync(filePath).mtimeMs));
  return import(fileUrl.href);
}

function parseJsonConfig(content: string, filePath: string): ResolvedVizeConfig {
  try {
    return normalizeLoadedConfig(JSON.parse(content));
  } catch (error) {
    throw new Error(`Failed to parse vize config JSON at ${filePath}: ${getErrorMessage(error)}`);
  }
}

function normalizeLoadedConfig(config: unknown): ResolvedVizeConfig {
  const normalized = stripNullish(config);
  if (Array.isArray(normalized)) {
    return normalizeConfigEntries(normalized as CompatVizeConfigEntry[]);
  }

  return normalizeConfigObject((normalized ?? {}) as CompatVizeConfig);
}

function normalizeConfigObject(config: CompatVizeConfig): ResolvedVizeConfig {
  const { entries: rawEntries, ...rootConfig } = normalizeConfigAliases(config) as VizeConfig & {
    entries?: CompatVizeConfigEntry[];
  };
  const rootEntry = rootConfig as VizeConfigEntry;
  const entries = [
    ...(isEmptyConfigEntry(rootEntry) ? [] : [rootEntry]),
    ...(rawEntries ?? []).map((entry) => normalizeConfigAliases(entry) as VizeConfigEntry),
  ];

  return {
    ...rootEntry,
    entries,
  };
}

function normalizeConfigEntries(entries: CompatVizeConfigEntry[]): ResolvedVizeConfig {
  const normalizedEntries = entries.map(
    (entry) => normalizeConfigAliases(entry) as VizeConfigEntry,
  );
  const globalConfig = mergeConfigEntries(normalizedEntries.filter(isGlobalConfigEntry));

  return {
    ...globalConfig,
    entries: normalizedEntries,
  };
}

function mergeConfigEntries(entries: VizeConfigEntry[]): VizeConfigEntry {
  const result: Record<string, unknown> = {};
  for (const entry of entries) {
    deepMerge(result, stripEntryMetadata(entry));
  }
  return result as VizeConfigEntry;
}

function stripEntryMetadata(entry: VizeConfigEntry): Partial<VizeConfigEntry> {
  const { name, basePath, files, ignores, extends: extendsConfig, ...config } = entry;
  void name;
  void basePath;
  void files;
  void ignores;
  void extendsConfig;
  return config;
}

function deepMerge(target: Record<string, unknown>, source: Record<string, unknown>): void {
  for (const [key, value] of Object.entries(source)) {
    if (value === undefined) {
      continue;
    }

    const current = target[key];
    if (isPlainObject(current) && isPlainObject(value)) {
      deepMerge(current, value);
    } else {
      target[key] = value;
    }
  }
}

function isGlobalConfigEntry(entry: VizeConfigEntry): boolean {
  return entry.basePath === undefined && entry.files === undefined && entry.ignores === undefined;
}

function isEmptyConfigEntry(entry: VizeConfigEntry): boolean {
  return Object.keys(entry).length === 0;
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function stripNullish(value: unknown): unknown {
  if (value === null) {
    return undefined;
  }

  if (Array.isArray(value)) {
    return value.map((entry) => stripNullish(entry)).filter((entry) => entry !== undefined);
  }

  if (typeof value === "object" && value !== null) {
    const result: Record<string, unknown> = {};
    for (const [key, entry] of Object.entries(value)) {
      const normalizedEntry = stripNullish(entry);
      if (normalizedEntry !== undefined) {
        result[key] = normalizedEntry;
      }
    }
    return result;
  }

  return value;
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  return String(error);
}

/**
 * Normalize GlobalTypesConfig shorthand strings to GlobalTypeDeclaration objects
 */
export function normalizeGlobalTypes(
  config: GlobalTypesConfig,
): Record<string, GlobalTypeDeclaration> {
  const resolvedConfig =
    "types" in config &&
    typeof config.types === "object" &&
    config.types !== null &&
    !Array.isArray(config.types)
      ? config.types
      : config;

  const result: Record<string, GlobalTypeDeclaration> = {};
  for (const [key, value] of Object.entries(resolvedConfig)) {
    if (typeof value === "string") {
      result[key] = { type: value };
    } else {
      result[key] = value;
    }
  }
  return result;
}

function normalizeConfigAliases(config: CompatVizeConfig): VizeConfig {
  if (config.lsp === undefined) {
    return config;
  }

  const { lsp, ...rest } = config;
  if (config.languageServer !== undefined) {
    return rest;
  }

  return {
    ...rest,
    languageServer: lsp,
  };
}
