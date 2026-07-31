import fs from "node:fs";
import path from "node:path";

import {
  dependencyNames,
  OXLINT_CONFIG_FILES,
  parsePackageJson,
  readRequiredFile,
  VIZE_CONFIG_FILES,
} from "../setup/config.js";

export const NUXT_CONFIG_FILES = [
  "nuxt.config.ts",
  "nuxt.config.mts",
  "nuxt.config.js",
  "nuxt.config.mjs",
] as const;

export const VITE_CONFIG_FILES = [
  "vite.config.ts",
  "vite.config.mts",
  "vite.config.js",
  "vite.config.mjs",
] as const;

export type PackageManager = "pnpm" | "npm" | "yarn" | "bun";

/**
 * Bundler integration `init` can wire up.
 *
 * Nuxt outranks Vite because a Nuxt project owns its own Vite instance: adding
 * `@vizejs/vite-plugin` to a Nuxt-managed Vite config would fight the module.
 */
export type Framework = "nuxt" | "vite" | "none";

export interface ProjectDetection {
  readonly root: string;
  readonly packageManager: PackageManager | null;
  readonly framework: Framework;
  readonly nuxtConfig: string | null;
  readonly viteConfigs: readonly string[];
  readonly usesVitePlus: boolean;
  readonly typescript: boolean;
  readonly tsconfig: string | null;
  readonly vizeConfig: string | null;
  readonly oxlintConfig: string | null;
  readonly hasVitePlusLintBlock: boolean;
  readonly hasVizeVitePlugin: boolean;
  readonly hasVizeNuxtModule: boolean;
  readonly dependencies: ReadonlySet<string>;
  readonly scripts: Readonly<Record<string, string>>;
  readonly vscodeRecommendsVize: boolean;
}

/**
 * Package-manager detection.
 *
 * Deliberately mirrors `detect_package_manager` in
 * `crates/vize_canon/src/batch/error.rs`, including the lockfile priority order
 * and the `packageManager` prefix fallback. The Rust side suggests an install
 * command in its corsa-not-found message; if the two ever disagreed, a user
 * would be told to run `pnpm add` by one half of the toolchain and `npm install`
 * by the other.
 */
export function detectPackageManager(root: string): PackageManager | null {
  const exists = (name: string): boolean => fs.existsSync(path.join(root, name));
  if (exists("pnpm-lock.yaml")) {
    return "pnpm";
  }
  if (exists("bun.lockb") || exists("bun.lock")) {
    return "bun";
  }
  if (exists("yarn.lock")) {
    return "yarn";
  }
  if (exists("package-lock.json")) {
    return "npm";
  }
  return detectPackageManagerField(root);
}

function detectPackageManagerField(root: string): PackageManager | null {
  let source: string;
  try {
    source = fs.readFileSync(path.join(root, "package.json"), "utf8");
  } catch {
    return null;
  }
  let field: unknown;
  try {
    field = (JSON.parse(source) as { packageManager?: unknown }).packageManager;
  } catch {
    return null;
  }
  if (typeof field !== "string") {
    return null;
  }
  for (const candidate of ["pnpm", "yarn", "bun", "npm"] as const) {
    if (field.startsWith(candidate)) {
      return candidate;
    }
  }
  return null;
}

/**
 * Applies an explicit `--vite` / `--nuxt` choice over what detection concluded.
 *
 * Overriding the framework rather than branching later keeps one code path: the
 * planner, the prompt and the printed detection summary all see the same answer,
 * so the summary cannot claim Vite while the plan configures Nuxt.
 */
export function withFramework(
  detection: ProjectDetection,
  framework: Framework | null,
): ProjectDetection {
  return framework === null || framework === detection.framework
    ? detection
    : { ...detection, framework };
}

export function detectProject(root: string): ProjectDetection {
  const packagePath = path.join(root, "package.json");
  const packageSource = readRequiredFile(packagePath, "No package.json found");
  const packageJson = parsePackageJson(packagePath, packageSource);
  const dependencies = dependencyNames(packageJson);
  const scripts = readScripts(packageJson);

  const nuxtConfig = findExisting(root, NUXT_CONFIG_FILES);
  const viteConfigs = VITE_CONFIG_FILES.filter((candidate) =>
    fs.existsSync(path.join(root, candidate)),
  );
  const viteSource = viteConfigs.length === 1 ? readFile(root, viteConfigs[0]!) : null;
  const nuxtSource = nuxtConfig === null ? null : readFile(root, nuxtConfig);

  return {
    root,
    packageManager: detectPackageManager(root),
    framework: detectFramework(nuxtConfig, viteConfigs, dependencies),
    nuxtConfig,
    viteConfigs,
    usesVitePlus: detectVitePlus(dependencies, viteSource, scripts),
    typescript: dependencies.has("typescript") || fs.existsSync(path.join(root, "tsconfig.json")),
    tsconfig: fs.existsSync(path.join(root, "tsconfig.json")) ? "tsconfig.json" : null,
    vizeConfig: findExisting(root, VIZE_CONFIG_FILES),
    oxlintConfig: findExisting(root, OXLINT_CONFIG_FILES),
    hasVitePlusLintBlock: viteSource !== null && viteSource.includes("oxlint-plugin-vize"),
    hasVizeVitePlugin: viteSource !== null && viteSource.includes("@vizejs/vite-plugin"),
    hasVizeNuxtModule: nuxtSource !== null && nuxtSource.includes("@vizejs/nuxt"),
    dependencies,
    scripts,
    vscodeRecommendsVize: detectVscodeRecommendation(root),
  };
}

function detectFramework(
  nuxtConfig: string | null,
  viteConfigs: readonly string[],
  dependencies: ReadonlySet<string>,
): Framework {
  if (nuxtConfig !== null || dependencies.has("nuxt")) {
    return "nuxt";
  }
  return viteConfigs.length > 0 ? "vite" : "none";
}

/**
 * Whether the project's lint command is `vp lint` rather than the `oxlint` binary.
 *
 * This single boolean decides which file `init` must write the Oxlint
 * configuration into, so it is deliberately generous: a project is treated as a
 * Vite+ project if the dependency is declared, if its Vite config imports from
 * `vite-plus`, or if any script invokes `vp`. Guessing "plain Oxlint" for a
 * Vite+ project is the failure that #3389 documented — `vp lint` would ignore
 * `.oxlintrc.json` and report zero Vize diagnostics while exiting 0.
 */
function detectVitePlus(
  dependencies: ReadonlySet<string>,
  viteSource: string | null,
  scripts: Readonly<Record<string, string>>,
): boolean {
  if (dependencies.has("vite-plus")) {
    return true;
  }
  if (viteSource !== null && /from\s+["']vite-plus["']/u.test(viteSource)) {
    return true;
  }
  return Object.values(scripts).some((command) => /(?:^|[\s&|;])vpx?(?:\s|$)/u.test(command));
}

function detectVscodeRecommendation(root: string): boolean {
  let source: string;
  try {
    source = fs.readFileSync(path.join(root, ".vscode", "extensions.json"), "utf8");
  } catch {
    return false;
  }
  return source.includes("ubugeeei.vize");
}

function readScripts(packageJson: Record<string, unknown>): Record<string, string> {
  const scripts = packageJson.scripts;
  if (typeof scripts !== "object" || scripts === null || Array.isArray(scripts)) {
    return {};
  }
  const entries: Record<string, string> = {};
  for (const [name, command] of Object.entries(scripts)) {
    if (typeof command === "string") {
      entries[name] = command;
    }
  }
  return entries;
}

function findExisting(root: string, candidates: readonly string[]): string | null {
  return candidates.find((candidate) => fs.existsSync(path.join(root, candidate))) ?? null;
}

function readFile(root: string, relative: string): string {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
