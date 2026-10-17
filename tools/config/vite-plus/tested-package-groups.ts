import type { PackagePath } from "./task-helpers.ts";

export type TestedPackageGroup = {
  readonly name: string;
  readonly packages: readonly PackagePath[];
  readonly prebuilt: readonly PackagePath[];
};

/**
 * Pull-request CI runs `testedPackages` as one parallel job per group.
 *
 * The groups partition `testedPackages` exactly and are balanced by the
 * package times measured in PR run 38355807070 (Vite plugin 88 s, UI 57 s,
 * Oxlint 44 s). Each group keeps the root native preparation, the package
 * test preparation wrapper and one package at a time.
 *
 * A group contains every tested workspace dependency of its packages, except
 * the ones in `prebuilt`. Those dependencies' tests start with the same
 * `vp pack` as their `build` script, so building them first gives dependents
 * the bundles they read when all packages ran in one job.
 */
export const testedPackageGroups = [
  {
    name: "builders",
    packages: [
      "./npm/cli",
      "./npm/builder/vite",
      "./npm/builder/vite-musea",
      "./npm/builder/unplugin",
      "./npm/builder/rspack",
      "./npm/framework/nuxt-lint-config",
      "./npm/framework/nuxt",
      "./npm/framework/musea-nuxt",
    ],
    prebuilt: ["./npm/oxlint"],
  },
  {
    name: "ui",
    packages: [
      "./npm/mcp-musea",
      "./npm/fresco",
      "./npm/compose/core",
      "./npm/compose/data",
      "./npm/compose/router",
      "./npm/compose/state",
      "./npm/ui",
      "./npm/marquette",
    ],
    prebuilt: [],
  },
  {
    name: "oxlint",
    packages: ["./npm/oxlint"],
    prebuilt: [],
  },
] as const satisfies readonly TestedPackageGroup[];
