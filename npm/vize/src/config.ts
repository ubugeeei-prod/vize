import * as fs from "node:fs";
import * as path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";
import { transform } from "oxc-transform";
import type {
  LanguageServerConfig,
  VizeConfig,
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

type CompatVizeConfig = VizeConfig & {
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
): Promise<VizeConfig | null> {
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

async function loadConfigFromDir(dir: string, env?: ConfigEnv): Promise<VizeConfig | null> {
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
): Promise<VizeConfig | null> {
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

async function loadConfigFile(filePath: string, env?: ConfigEnv): Promise<VizeConfig | null> {
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
          return candidate;
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

function loadPklConfig(filePath: string): VizeConfig | null {
  const pklBin = findPklBinary();
  if (!pklBin) {
    console.warn(
      "[vize] pkl CLI not found. Install @pkl-community/pkl or add pkl to PATH. " +
        "Falling back to the next config format.",
    );
    return null;
  }

  try {
    const output = execFileSync(pklBin, ["eval", "-f", "json", filePath], {
      cwd: path.dirname(filePath),
      encoding: "utf-8",
      stdio: ["ignore", "pipe", "pipe"],
      timeout: 30_000,
    });
    return parseJsonConfig(output, filePath);
  } catch (error) {
    console.warn(`[vize] Failed to evaluate ${filePath}: ${getErrorMessage(error)}`);
    return null;
  }
}

async function resolveConfigExport(
  exported: UserConfigExport,
  env?: ConfigEnv,
): Promise<VizeConfig> {
  if (typeof exported === "function") {
    return normalizeLoadedConfig(await exported(env ?? DEFAULT_CONFIG_ENV));
  }

  return normalizeLoadedConfig(exported);
}

async function loadTypeScriptConfig(filePath: string, env?: ConfigEnv): Promise<VizeConfig> {
  const source = fs.readFileSync(filePath, "utf-8");
  const result = transform(filePath, source, {
    typescript: {
      onlyRemoveTypeImports: true,
    },
  });

  const tempFile = path.join(
    path.dirname(filePath),
    `.vize-config-${process.pid}-${Date.now()}.mjs`,
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

async function loadESMConfig(filePath: string, env?: ConfigEnv): Promise<VizeConfig> {
  const module = await importFresh(filePath);
  const exported: UserConfigExport = module.default || module;
  return resolveConfigExport(exported, env);
}

async function importFresh(filePath: string): Promise<Record<string, unknown>> {
  const fileUrl = pathToFileURL(filePath);
  fileUrl.searchParams.set("t", String(fs.statSync(filePath).mtimeMs));
  return import(fileUrl.href);
}

function parseJsonConfig(content: string, filePath: string): VizeConfig {
  try {
    return normalizeLoadedConfig(JSON.parse(content));
  } catch (error) {
    throw new Error(`Failed to parse vize config JSON at ${filePath}: ${getErrorMessage(error)}`);
  }
}

function normalizeLoadedConfig(config: unknown): VizeConfig {
  const normalized = stripNullish(config);
  return normalizeConfigAliases((normalized ?? {}) as CompatVizeConfig);
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
