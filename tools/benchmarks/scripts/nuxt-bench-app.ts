/**
 * Dependencies of the tool benchmark's Nuxt build surface. Nuxt and Vue come
 * from the pinned npm lockfile in tools/benchmarks/nuxt-app, resolved the way
 * a user's app resolves them (no workspace overrides). `@vizejs/nuxt` is this
 * commit's build, installed beside them as package files.
 */

import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { installPackageFiles } from "./install-package-files.ts";

const benchDir = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(benchDir, "..", "..", "..");

export const NUXT_APP_DIR = resolve(benchDir, "..", "nuxt-app");
export const VIZE_NUXT_PACKAGE_DIR = join(rootDir, "npm", "framework", "nuxt");

export type NuxtBenchInstall = {
  modules: string;
  nuxtBin: string;
  dependencies: Record<string, string>;
};

const packageVersion = (dir: string): string =>
  (JSON.parse(readFileSync(join(dir, "package.json"), "utf8")) as { version: string }).version;

/** Install `@vizejs/nuxt` into the pinned app; a missing install is an error. */
export function prepareNuxtBenchInstall(
  appDir = NUXT_APP_DIR,
  moduleDir = VIZE_NUXT_PACKAGE_DIR,
): NuxtBenchInstall {
  const modules = join(appDir, "node_modules");
  const nuxtBin = join(modules, ".bin", "nuxt");
  if (!existsSync(nuxtBin)) {
    throw new Error(
      `Nuxt benchmark app is not installed (${nuxtBin} missing). Run \`npm ci --prefix tools/benchmarks/nuxt-app\` first.`,
    );
  }
  installPackageFiles(moduleDir, modules, "@vizejs/nuxt");
  const manifest = JSON.parse(readFileSync(join(appDir, "package.json"), "utf8")) as {
    dependencies: Record<string, string>;
  };
  return {
    modules,
    nuxtBin,
    dependencies: {
      ...Object.fromEntries(
        Object.keys(manifest.dependencies).map((name) => [
          name,
          packageVersion(join(modules, name)),
        ]),
      ),
      "@vizejs/nuxt": packageVersion(moduleDir),
    },
  };
}
