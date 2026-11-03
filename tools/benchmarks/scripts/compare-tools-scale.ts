/**
 * The type-check and Nuxt build surfaces run on generated scale projects
 * (scale-project-generator.ts) rather than on the flat SFC corpus: a Vue
 * project of generic layers forwarding scoped slots, kebab-case emits,
 * `v-model` chains, generic kit components, global components and deep
 * import trees, and the same features as a Nuxt 4 app with one route each.
 *
 * Each surface has two tiers: the headline (10,000 SFCs on Blacksmith) and a
 * smaller one (a quarter of it, at least the generator's minimum). The small
 * type-check tier carries every lane and checker setting; the headline runs
 * one lane per tool. Generator plants are off: the planted-diagnostic gate
 * (typecheck-readiness.mjs) plants its own copy and requires every lane to
 * report it.
 */

import { cpSync, existsSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import type { NuxtBenchInstall } from "./nuxt-bench-app.ts";
import { linkColdNodeModules } from "./nuxt-build-cache.ts";
import { generateScaleProject, minSfcs, type ScaleKind } from "./scale-project-generator.ts";
import { VIZE_NUXT_MODULE, nuxtConfig } from "./scale-project-nuxt.ts";

export type ScaleTier = { id: string; label: string; sfcs: number };
type Timing = { ms: number; cpuMs: number | null };
type RunCommand = (
  binary: string,
  args: string[],
  options: { cwd: string; allowNonZeroExit: boolean; env: Record<string, string> },
) => Timing;
type Variant = { id: string; label: string; files: number; measure: () => Timing };

/** The smaller tier of a headline size. */
export const smallTierSfcs = (sfcs: number, kind: ScaleKind): number =>
  Math.max(minSfcs(kind), Math.round(sfcs / 4));

export function typecheckTiers(sfcs: number, smallSfcs: number): [ScaleTier, ScaleTier] {
  return [
    { id: "check", label: "Type check", sfcs },
    { id: "check-small", label: "Type check (small tier)", sfcs: smallSfcs },
  ];
}

export function nuxtTiers(sfcs: number, smallSfcs: number): [ScaleTier, ScaleTier] {
  return [
    { id: "nuxt", label: "Nuxt SPA build (end-to-end)", sfcs },
    { id: "nuxt-small", label: "Nuxt SPA build (end-to-end, small tier)", sfcs: smallSfcs },
  ];
}

const sfcBytes = (files: Map<string, string>): number =>
  [...files]
    .filter(([path]) => path.endsWith(".vue"))
    .reduce((sum, [, text]) => sum + Buffer.byteLength(text), 0);

/** Write the Vue kind of `tier` under `workRoot`; the caller installs packages. */
export function writeScaleCheckProject(
  workRoot: string,
  tier: ScaleTier,
): { dir: string; bytes: number } {
  const dir = join(workRoot, tier.id);
  const project = generateScaleProject({ sfcs: tier.sfcs, kind: "vue", plants: false, dir });
  return { dir, bytes: sfcBytes(project.files) };
}

const NUXT_BUILD_ENV = { CI: "1", NITRO_PRESET: "node-server", NUXT_TELEMETRY_DISABLED: "1" };

/** A client-only build still ships the node-server entry, and every route is its own chunk. */
export function assertNuxtBuilt(dir: string, routes: number): void {
  const assets = join(dir, ".output", "public", "_nuxt");
  const scripts = existsSync(assets)
    ? readdirSync(assets).filter((file) => file.endsWith(".js")).length
    : 0;
  if (!existsSync(join(dir, ".output", "server", "index.mjs")) || scripts < routes) {
    throw new Error(
      `nuxt build in ${dir} emitted ${scripts} client scripts for ${routes} routes${existsSync(join(dir, ".output", "server", "index.mjs")) ? "" : " and no server entry"}`,
    );
  }
}

/**
 * `nuxt build` of the Nuxt kind with Nuxt's default compiler and with
 * `@vizejs/nuxt` at its defaults. Every run builds a fresh copy against cold
 * shared caches, and must produce the server entry and one chunk per route.
 */
export async function measureScaleNuxt<Surface>({
  tier,
  workRoot,
  install,
  runCommand,
  measureVariants,
  createSurface,
  options,
}: {
  tier: ScaleTier;
  workRoot: string;
  install: NuxtBenchInstall;
  runCommand: RunCommand;
  measureVariants: (variants: Variant[], options: unknown) => Promise<unknown[]>;
  createSurface: (surface: Record<string, unknown>) => Surface;
  options: unknown;
}): Promise<Surface> {
  const template = join(workRoot, tier.id, "template");
  const project = generateScaleProject({
    sfcs: tier.sfcs,
    kind: "nuxt",
    plants: false,
    dir: template,
  });
  let invocation = 0;
  const build = (label: string, useVize: boolean): Timing => {
    const dir = join(workRoot, tier.id, `${label}-${String(++invocation).padStart(4, "0")}`);
    rmSync(dir, { recursive: true, force: true });
    cpSync(template, dir, { recursive: true });
    writeFileSync(join(dir, "nuxt.config.ts"), nuxtConfig(useVize ? [VIZE_NUXT_MODULE] : []));
    writeFileSync(
      join(dir, "package.json"),
      `${JSON.stringify({ private: true, type: "module", dependencies: install.dependencies }, null, 2)}\n`,
    );
    linkColdNodeModules(dir, [install.modules], label);
    try {
      const timing = runCommand(install.nuxtBin, ["build"], {
        cwd: dir,
        allowNonZeroExit: false,
        env: NUXT_BUILD_ENV,
      });
      assertNuxtBuilt(dir, project.features);
      return timing;
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  };
  const variants: Variant[] = [
    {
      id: "nuxt-default",
      label: "Nuxt default compiler",
      files: tier.sfcs,
      measure: () => build("default", false),
    },
    {
      id: "vize-nuxt",
      label: "@vizejs/nuxt",
      files: tier.sfcs,
      measure: () => build("vize", true),
    },
  ];
  try {
    return createSurface({
      id: tier.id,
      label: tier.label,
      files: tier.sfcs,
      bytes: sfcBytes(project.files),
      variants: await measureVariants(variants, options),
      baselineId: "nuxt-default",
      vizeSingleId: null,
      vizeMaxId: "vize-nuxt",
    });
  } finally {
    rmSync(template, { recursive: true, force: true });
  }
}
