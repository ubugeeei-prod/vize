import type { PackagePath, TaskInput } from "./task-helpers.ts";

/**
 * Packages whose TypeScript, Vite, or package-level checks are part of the root
 * workspace quality gate.
 *
 * The list intentionally includes runnable examples and the playground because
 * those surfaces catch integration drift that package-local unit tests cannot
 * see. Each path is a typed package-relative literal, which keeps task filters
 * predictable and prevents accidental shell fragments from entering task
 * commands.
 */
export const checkedPackages = [
  "./npm/vize",
  "./npm/vite-plugin-vize",
  "./npm/oxlint-plugin-vize",
  "./npm/vite-plugin-musea",
  "./npm/unplugin-vize",
  "./npm/rspack-vize-plugin",
  "./npm/nuxt",
  "./npm/musea-nuxt",
  "./npm/musea-mcp-server",
  "./npm/fresco",
  "./npm/vite-plugin-vize/example",
  "./npm/rspack-vize-plugin/example",
  "./examples/vite-musea",
  "./examples/oxlint-vize",
  "./playground",
] satisfies PackagePath[];

/**
 * Packages that need their own direct package-manager scripts instead of a
 * filtered Vite+ run.
 */
export const directCheckPackages = [
  "./examples/vite-musea",
  "./playground",
] satisfies PackagePath[];

const directCheckPackageSet = new Set<PackagePath>(directCheckPackages);

export const checkedPackagesViaVpRun = checkedPackages.filter(
  (pkg) => !directCheckPackageSet.has(pkg),
);

/**
 * CI excludes the oxlint example from the aggregate check because that example
 * intentionally demonstrates a failing lint script.
 */
export const ciCheckedPackages = checkedPackagesViaVpRun.filter(
  (pkg) => pkg !== "./examples/oxlint-vize",
);

export const packedPackages = [
  "./npm/vize",
  "./npm/vite-plugin-vize",
  "./npm/oxlint-plugin-vize",
  "./npm/vite-plugin-musea",
  "./npm/unplugin-vize",
  "./npm/rspack-vize-plugin",
  "./npm/nuxt",
  "./npm/musea-nuxt",
  "./npm/musea-mcp-server",
  "./npm/fresco",
] satisfies PackagePath[];

export const testedPackages = [
  "./npm/vite-plugin-vize",
  "./npm/oxlint-plugin-vize",
  "./npm/vite-plugin-musea",
  "./npm/unplugin-vize",
  "./npm/rspack-vize-plugin",
  "./npm/nuxt",
  "./npm/musea-mcp-server",
] satisfies PackagePath[];

export const floatingPromiseTestPatterns = ["tests/**/*.ts"];

const taskConfigInputs = ["vite.config.ts", "tools/vite-plus/**"];

/**
 * Cache inputs for the root task catalog.
 *
 * These groups are deliberately broad enough to protect correctness across the
 * monorepo, but still narrower than "everything" so repeated CI and local runs
 * avoid unnecessary rebuilds. The Vite+ task modules are included explicitly
 * because changing orchestration should invalidate cached task results just as
 * surely as changing package source code.
 */
export const cacheInputs = {
  workspace: [".node-version", "package.json", ...taskConfigInputs, "pnpm-lock.yaml"],
  jsChecks: [
    ".node-version",
    "package.json",
    ...taskConfigInputs,
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
    "npm/**/package.json",
    "npm/**/vite.config.ts",
    "npm/**/rspack.config.ts",
    "npm/**/src/**",
    "examples/**/package.json",
    "examples/**/vite.config.ts",
    "examples/**/playwright.config.ts",
    "examples/**/src/**",
    "playground/package.json",
    "playground/vite*.ts",
    "playground/playwright.config.ts",
    "playground/src/**",
    "playground/e2e/**",
  ],
  rust: [
    ".node-version",
    "package.json",
    ...taskConfigInputs,
    "Cargo.toml",
    "Cargo.lock",
    "crates/**",
    "tests/**",
    "tools/**",
  ],
  e2e: [
    ".node-version",
    "package.json",
    ...taskConfigInputs,
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
    "tests/package.json",
    "tests/app/**",
    "tests/_helpers/**",
    "tests/_fixtures/**",
    "tests/snapshots/**",
    "npm/vize*/**",
    "npm/vite-plugin-vize/**",
    "npm/nuxt/**",
    "npm/vite-plugin-musea/**",
    "npm/musea-nuxt/**",
  ],
} satisfies Record<string, TaskInput>;
