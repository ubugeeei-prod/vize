import fs from "node:fs";
import path from "node:path";

export const VIZE_CONFIG_FILES = [
  "vize.config.pkl",
  "vize.config.ts",
  "vize.config.js",
  "vize.config.mjs",
  "vize.config.json",
] as const;

/** Config filenames Oxlint 1.64 auto-discovers. */
export const DISCOVERED_OXLINT_CONFIG_FILES = [
  ".oxlintrc.json",
  ".oxlintrc.jsonc",
  "oxlint.config.ts",
] as const;

/**
 * All plausible Oxlint config filenames, including names the binary ignores.
 *
 * Project detection keeps the ignored names so `vize init` can explain why it
 * will not preserve them. Setup must use `DISCOVERED_OXLINT_CONFIG_FILES`.
 */
export const OXLINT_CONFIG_FILES = [
  ...DISCOVERED_OXLINT_CONFIG_FILES,
  "oxlint.config.mts",
  "oxlint.config.js",
  "oxlint.config.mjs",
  "oxlint.config.cjs",
  "oxlint.config.cts",
] as const;

export const REQUIRED_DEV_DEPENDENCIES = [
  "vize",
  "@vizejs/vite-plugin",
  "@vizejs/vite-plugin-musea",
  "oxlint",
  "oxlint-plugin-vize",
] as const;

export const DEFAULT_SCRIPTS = {
  "vize:build": "vize build src",
  "vize:fmt": "vize fmt --check src",
  "vize:fmt:fix": "vize fmt --write src",
  "vize:lint": "vize lint --preset happy-path --max-warnings 0 src",
  // No positional input: the default command must check the complete tsconfig
  // project graph, including root files and referenced projects outside src.
  "vize:check": "vize check",
  "vize:musea": "vize musea",
  "vize:ready": "vize ready src",
} as const;

export const DEFAULT_VIZE_CONFIG = `import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    templateSyntax: "standard",
  },
  linter: {
    preset: "happy-path",
  },
  typeChecker: {
    enabled: true,
    strict: true,
    jsxTypecheck: true,
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
`;

export const DEFAULT_OXLINT_CONFIG = `import { defineConfig } from "oxlint";
import { configs } from "oxlint-plugin-vize";

export default defineConfig({
  plugins: ["vue"],
  jsPlugins: ["oxlint-plugin-vize"],
  settings: {
    vize: {
      preset: "happy-path",
      helpLevel: "short",
    },
  },
  rules: configs.happyPath,
});
`;

type JsonObject = Record<string, unknown>;

export interface PlannedFile {
  readonly filename: string;
  readonly source: string;
}

export function readRequiredFile(filename: string, message: string): string {
  try {
    return fs.readFileSync(filename, "utf8");
  } catch (error) {
    if (isNodeError(error) && error.code === "ENOENT") {
      throw new Error(`${message}: ${filename}`, { cause: error });
    }
    throw error;
  }
}

export function parsePackageJson(filename: string, source: string): JsonObject {
  try {
    const parsed = JSON.parse(source) as unknown;
    if (!isJsonObject(parsed)) {
      throw new Error("package.json must contain an object");
    }
    return parsed;
  } catch (error) {
    throw new Error(`Invalid package.json: ${filename}`, { cause: error });
  }
}

export function detectJsonIndent(source: string): string | number {
  const match = source.match(/^[\t ]+(?=")/mu);
  return match?.[0] ?? 2;
}

export function dependencyNames(packageJson: JsonObject): Set<string> {
  const names = new Set<string>();
  for (const field of ["dependencies", "devDependencies", "optionalDependencies"] as const) {
    const dependencies = packageJson[field];
    if (!isJsonObject(dependencies)) {
      continue;
    }
    for (const name of Object.keys(dependencies)) {
      names.add(name);
    }
  }
  return names;
}

export function addDefaultScripts(packageJson: JsonObject): {
  readonly addedScripts: string[];
  readonly preservedScripts: string[];
} {
  if (packageJson.scripts !== undefined && !isJsonObject(packageJson.scripts)) {
    throw new Error("package.json scripts must contain an object");
  }

  const scripts = (packageJson.scripts ?? {}) as JsonObject;
  const addedScripts: string[] = [];
  const preservedScripts: string[] = [];
  for (const [name, command] of Object.entries(DEFAULT_SCRIPTS)) {
    if (name in scripts) {
      preservedScripts.push(name);
      continue;
    }
    scripts[name] = command;
    addedScripts.push(name);
  }
  if (addedScripts.length > 0) {
    packageJson.scripts = scripts;
  }
  return { addedScripts, preservedScripts };
}

export function planGeneratedConfig(
  root: string,
  candidates: readonly string[],
  generatedName: string,
  source: string,
  plannedFiles: PlannedFile[],
  createdFiles: string[],
  preservedFiles: string[],
): void {
  const existing = candidates.find((candidate) => fs.existsSync(path.join(root, candidate)));
  if (existing) {
    preservedFiles.push(existing);
    return;
  }
  plannedFiles.push({ filename: path.join(root, generatedName), source });
  createdFiles.push(generatedName);
}

export function atomicWriteFile(filename: string, source: string): void {
  const temporary = path.join(
    path.dirname(filename),
    `.${path.basename(filename)}.${process.pid}.${Date.now()}.tmp`,
  );
  try {
    fs.writeFileSync(temporary, source, { encoding: "utf8", flag: "wx" });
    fs.renameSync(temporary, filename);
  } finally {
    fs.rmSync(temporary, { force: true });
  }
}

function isJsonObject(value: unknown): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isNodeError(value: unknown): value is NodeJS.ErrnoException {
  return value instanceof Error;
}
