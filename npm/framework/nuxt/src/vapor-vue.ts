import { readFileSync, realpathSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";

import type { VizeNuxtCompilerOptions } from "./compiler-options.ts";

type ResolvedVueCopy = {
  version: string;
  directory: string;
};

/**
 * Fail before Vite when Vapor would compile an SFC against Vue older than 3.6.
 * `3.6.0-rc.12` is new enough; do not apply semver's prerelease ordering.
 */
export function assertNuxtVaporVue(
  rootDir: string,
  compiler: VizeNuxtCompilerOptions,
): VizeNuxtCompilerOptions {
  if (compiler.vapor !== true) return compiler;

  const tooOld = vueCopiesForVapor(rootDir, compiler.handleNodeModulesVue !== false).filter(
    (copy) => !vueVersionSupportsVapor(copy.version),
  );
  if (tooOld.length === 0) return compiler;

  const details = tooOld.map((copy) => `vue@${copy.version} at ${copy.directory}`).join("; ");
  throw new Error(`@vizejs/nuxt: compiler.vapor requires Vue >= 3.6. Resolved ${details}.`);
}

function vueCopiesForVapor(rootDir: string, includeNuxtPackageVue: boolean): ResolvedVueCopy[] {
  const appPackageJson = join(rootDir, "package.json");
  const copies = [readVueCopy(resolveFromPackage(appPackageJson, "vue/package.json"))];
  if (includeNuxtPackageVue) {
    const nuxtPackageJson = resolveFromPackage(appPackageJson, "nuxt/package.json");
    copies.push(readVueCopy(resolveFromPackage(nuxtPackageJson, "vue/package.json")));
  }
  return uniqueVueCopies(copies);
}

function resolveFromPackage(packageJsonPath: string, specifier: string): string {
  try {
    return createRequire(packageJsonPath).resolve(specifier);
  } catch (error) {
    throw new Error(
      `@vizejs/nuxt: compiler.vapor requires Vue >= 3.6, but ${specifier} could not be resolved from ${packageJsonPath}.`,
      { cause: error },
    );
  }
}

function readVueCopy(packageJsonPath: string): ResolvedVueCopy {
  let parsed: { version?: unknown };
  try {
    parsed = JSON.parse(readFileSync(packageJsonPath, "utf8")) as { version?: unknown };
  } catch (error) {
    throw new Error(
      `@vizejs/nuxt: compiler.vapor requires Vue >= 3.6, but ${packageJsonPath} could not be read.`,
      { cause: error },
    );
  }
  if (typeof parsed.version !== "string" || parsed.version.length === 0) {
    throw new Error(
      `@vizejs/nuxt: compiler.vapor requires Vue >= 3.6, but ${packageJsonPath} has no version.`,
    );
  }
  return { version: parsed.version, directory: realpathSync(dirname(packageJsonPath)) };
}

function uniqueVueCopies(copies: ResolvedVueCopy[]): ResolvedVueCopy[] {
  const seen = new Set<string>();
  const unique: ResolvedVueCopy[] = [];
  for (const copy of copies) {
    if (seen.has(copy.directory)) continue;
    seen.add(copy.directory);
    unique.push(copy);
  }
  return unique;
}

/** Major above 3, or Vue 3.6+, including prereleases such as 3.6.0-rc.12. */
function vueVersionSupportsVapor(version: string): boolean {
  const match = /^v?(\d+)\.(\d+)(?:\.(\d+))?/.exec(version.trim());
  if (match == null) return false;
  const major = Number(match[1]);
  const minor = Number(match[2]);
  return major > 3 || (major === 3 && minor >= 6);
}
