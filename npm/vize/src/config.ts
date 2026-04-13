import * as fs from "node:fs";
import * as path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";
import { transform } from "oxc-transform";
import type {
  VizeConfig,
  LoadConfigOptions,
  UserConfigExport,
  ConfigEnv,
  GlobalTypesConfig,
  GlobalTypeDeclaration,
} from "./types/index.js";

export const CONFIG_FILE_NAMES = [
  "vize.config.ts",
  "vize.config.js",
  "vize.config.mjs",
  "vize.config.pkl",
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
    const configPath = findConfigFileAuto(root);
    if (!configPath) {
      return null;
    }
    return loadConfigFile(configPath, env);
  }

  const configPath = findConfigFileInDir(root);
  if (!configPath) {
    return null;
  }

  return loadConfigFile(configPath, env);
}

function findConfigFileInDir(dir: string): string | null {
  for (const name of CONFIG_FILE_NAMES) {
    const filePath = path.join(dir, name);
    if (fs.existsSync(filePath)) {
      return filePath;
    }
  }
  return null;
}

function findConfigFileAuto(startDir: string): string | null {
  let currentDir = path.resolve(startDir);

  while (true) {
    const configPath = findConfigFileInDir(currentDir);
    if (configPath) {
      return configPath;
    }

    const parentDir = path.dirname(currentDir);
    if (parentDir === currentDir) {
      return null;
    }

    currentDir = parentDir;
  }
}

async function loadConfigFile(filePath: string, env?: ConfigEnv): Promise<VizeConfig | null> {
  if (!fs.existsSync(filePath)) {
    return null;
  }

  const ext = path.extname(filePath);

  if (ext === ".json") {
    const content = fs.readFileSync(filePath, "utf-8");
    return parseJsonConfig(content, filePath);
  }

  if (ext === ".pkl") {
    return loadPklConfig(filePath);
  }

  if (ext === ".ts") {
    return loadTypeScriptConfig(filePath, env);
  }

  return loadESMConfig(filePath, env);
}

async function resolveConfigExport(
  exported: UserConfigExport,
  env?: ConfigEnv,
): Promise<VizeConfig> {
  if (typeof exported === "function") {
    return exported(env ?? DEFAULT_CONFIG_ENV);
  }

  return exported;
}

async function loadTypeScriptConfig(filePath: string, env?: ConfigEnv): Promise<VizeConfig> {
  const source = fs.readFileSync(filePath, "utf-8");
  const result = transform(filePath, source, {
    typescript: {
      onlyRemoveTypeImports: true,
    },
  });

  const tempFile = filePath.replace(/\.ts$/, `.temp.${Date.now()}.mjs`);
  fs.writeFileSync(tempFile, result.code);

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

function loadPklConfig(filePath: string): VizeConfig {
  try {
    const output = execFileSync("pkl", ["eval", "--format", "json", filePath], {
      cwd: path.dirname(filePath),
      encoding: "utf-8",
      stdio: ["ignore", "pipe", "pipe"],
    });
    return parseJsonConfig(output, filePath);
  } catch (error) {
    throw new Error(
      `Failed to evaluate PKL config at ${filePath}. Make sure the 'pkl' CLI is installed and on PATH. ${getErrorMessage(error)}`,
    );
  }
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
  return (normalized ?? {}) as VizeConfig;
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
  const result: Record<string, GlobalTypeDeclaration> = {};
  for (const [key, value] of Object.entries(config)) {
    if (typeof value === "string") {
      result[key] = { type: value };
    } else {
      result[key] = value;
    }
  }
  return result;
}
