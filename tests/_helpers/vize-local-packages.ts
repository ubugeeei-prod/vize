import { execFileSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

const HELPERS_DIR = path.dirname(fileURLToPath(import.meta.url));
const NPM_DIR = path.resolve(HELPERS_DIR, "../../npm");
const REPO_ROOT = path.resolve(HELPERS_DIR, "../..");

interface VizeLocalPackage {
  packageName: string;
  filter: string;
  dir: string;
  outputs: readonly string[];
  buildInFixtureSetup: boolean;
  linkIntoFixture: boolean;
}

/**
 * Local packages exposed to copied app fixtures, in dependency-first build order.
 *
 * Exported so the tooling regression can verify that this list remains closed
 * over runtime workspace dependencies as package manifests evolve.
 */
export const VIZE_LOCAL_PACKAGES = [
  {
    packageName: "@vizejs/native",
    filter: "@vizejs/native",
    dir: path.join(NPM_DIR, "native"),
    outputs: ["index.js"],
    buildInFixtureSetup: false,
    linkIntoFixture: true,
  },
  {
    packageName: "vize",
    filter: "vize",
    dir: path.join(NPM_DIR, "cli"),
    outputs: ["dist/index.mjs", "dist/config.mjs"],
    buildInFixtureSetup: true,
    linkIntoFixture: true,
  },
  {
    packageName: "@vizejs/vite-plugin",
    filter: "@vizejs/vite-plugin",
    dir: path.join(NPM_DIR, "builder/vite"),
    outputs: ["dist/index.mjs"],
    buildInFixtureSetup: true,
    linkIntoFixture: true,
  },
  {
    packageName: "@vizejs/nuxt-lint-config",
    filter: "@vizejs/nuxt-lint-config",
    dir: path.join(NPM_DIR, "framework/nuxt-lint-config"),
    outputs: ["dist/index.mjs"],
    buildInFixtureSetup: true,
    linkIntoFixture: true,
  },
  {
    packageName: "oxlint-plugin-vize",
    filter: "oxlint-plugin-vize",
    dir: path.join(NPM_DIR, "oxlint"),
    outputs: ["dist/index.mjs"],
    buildInFixtureSetup: true,
    linkIntoFixture: true,
  },
  {
    packageName: "@vizejs/vite-plugin-musea",
    filter: "@vizejs/vite-plugin-musea",
    dir: path.join(NPM_DIR, "builder/vite-musea"),
    outputs: ["dist/index.mjs", "dist/cli/index.mjs"],
    buildInFixtureSetup: true,
    linkIntoFixture: true,
  },
  {
    packageName: "@vizejs/nuxt",
    filter: "@vizejs/nuxt",
    dir: path.join(NPM_DIR, "framework/nuxt"),
    outputs: ["dist/index.mjs"],
    buildInFixtureSetup: true,
    linkIntoFixture: true,
  },
  {
    packageName: "@vizejs/musea-nuxt",
    filter: "@vizejs/musea-nuxt",
    dir: path.join(NPM_DIR, "framework/musea-nuxt"),
    outputs: ["dist/index.mjs"],
    buildInFixtureSetup: true,
    linkIntoFixture: true,
  },
] as const satisfies readonly VizeLocalPackage[];

const BUILT_VIZE_PACKAGES = new Set<string>();

function hasBuildOutputs(dir: string, outputs: readonly string[]): boolean {
  return outputs.every((output) => fs.existsSync(path.join(dir, output)));
}

export function ensureLocalVizePackagesBuilt(): void {
  for (const target of VIZE_LOCAL_PACKAGES) {
    if (!target.buildInFixtureSetup) continue;
    if (
      BUILT_VIZE_PACKAGES.has(target.packageName) &&
      hasBuildOutputs(target.dir, target.outputs)
    ) {
      continue;
    }

    if (!hasBuildOutputs(target.dir, target.outputs)) {
      console.log(`[vize:setup] building ${target.packageName}...`);
      execFileSync("npx", ["-y", "pnpm@10", "--filter", target.filter, "build"], {
        cwd: REPO_ROOT,
        stdio: "inherit",
        timeout: 300_000,
      });
    }

    BUILT_VIZE_PACKAGES.add(target.packageName);
  }
}

export function ensureSymlink(link: string, target: string): void {
  try {
    const stat = fs.lstatSync(link);
    if (stat.isSymbolicLink()) {
      try {
        if (fs.realpathSync(link) === fs.realpathSync(target)) {
          return;
        }
      } catch {
        // Replace a broken link.
      }
      fs.unlinkSync(link);
    } else {
      return;
    }
  } catch {
    // The link does not exist yet.
  }
  fs.symlinkSync(target, link, "dir");
}

export function createVizeSymlinks(nodeModulesDir: string): void {
  for (const target of VIZE_LOCAL_PACKAGES) {
    if (!target.linkIntoFixture) continue;
    const link = path.join(nodeModulesDir, ...target.packageName.split("/"));
    fs.mkdirSync(path.dirname(link), { recursive: true });
    ensureSymlink(link, target.dir);
  }
}
