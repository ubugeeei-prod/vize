/**
 * Generated Vue projects for type-check and build benchmarks at scale.
 *
 *   generateScaleProject({ sfcs: 10_000, kind: "vue" | "nuxt" | "monorepo", seed, plants, dir })
 *
 * Work per SFC is fixed by construction, so total cost should grow linearly
 * with N. Each feature is a page (a Nuxt panel), a 10-deep chain of generic
 * layers forwarding a scoped slot, kebab-case emits and `v-model:query`, and
 * a 9-deep widget chain (a six-branch `v-if` narrowing chain, a two-parameter
 * generic list, `defineModel` widgets). Kit components are generic and used
 * with inferred type arguments; six components are global. Pages form a
 * binary tree, so the deepest import paths grow with log N.
 *
 * - `vue`: one project, components imported, globals through a
 *   `GlobalComponents` augmentation.
 * - `monorepo`: the same features in composite packages with `references`,
 *   cross-package imports and one `components.d.ts` per feature package.
 * - `nuxt`: a Nuxt 4 `app/` tree (scale-project-nuxt.ts). Run `nuxt prepare`
 *   before checking; check `.nuxt/tsconfig.app.json`.
 *
 * Every PLANT_STRIDE-th feature (offset by the seed) and the last one carry
 * one type error of each PlantKind at a known line; `expectedPlants(plants,
 * lane)` gives what a checker lane must report. Output is a pure function of
 * the options, and building it is string work only (10k SFCs in well under a
 * second, plus the writes).
 */

import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, posix } from "node:path";

import { app, childPages, page, rowCount, widget } from "./scale-project-feature.ts";
import {
  NUXT_CONFIG,
  NUXT_LAYOUTS,
  NUXT_TSCONFIG,
  NUXT_APP_TSCONFIG,
  autoImportName,
  composable,
  indexPage,
  layout,
  nuxtApp,
  nuxtPlace,
  routePage,
} from "./scale-project-nuxt.ts";
import { PLANT_CODES, type Plant } from "./scale-project-plants.ts";
import {
  type Ctx,
  GLOBALS,
  KIT,
  LAYER_DEPTH,
  type Source,
  WIDGETS,
  featureDir,
  featureModel,
  globalComponent,
  globalDeclarations,
  kitComponent,
  layer,
  layerName,
  sharedTypes,
  widgetName,
} from "./scale-project-sfc.ts";

export { PLANT_CODES, PLANT_MULTIPLICITY, expectedPlants } from "./scale-project-plants.ts";
export type { Plant, PlantExpectation } from "./scale-project-plants.ts";

export type ScaleKind = "vue" | "nuxt" | "monorepo";
export type ScaleProjectOptions = {
  sfcs: number;
  kind: ScaleKind;
  /** Offsets which features carry plants and varies row counts; same seed, same bytes. */
  seed?: number;
  /** Plant the known type errors (default true). */
  plants?: boolean;
  /** Feature packages of a monorepo (default 6). */
  featurePackages?: number;
  /** Also write the files here, replacing the directory. */
  dir?: string;
};
export type ScaleProject = {
  kind: ScaleKind;
  sfcs: number;
  features: number;
  seed: number;
  /** Project-relative posix path to file text. */
  files: Map<string, string>;
  plants: Plant[];
  /** Monorepo package directories, dependencies first; empty otherwise. */
  packages: string[];
  /** The tsconfig a checker takes (a Nuxt one exists after `nuxt prepare`). */
  tsconfig: string;
};

/** SFCs outside features (kit, globals, root; Nuxt adds layouts and the index page) and per feature. */
const SHAPE: Record<ScaleKind, { fixed: number; feature: number }> = {
  vue: { fixed: KIT.length + GLOBALS.length + 1, feature: LAYER_DEPTH + 1 + WIDGETS },
  monorepo: { fixed: KIT.length + GLOBALS.length + 1, feature: LAYER_DEPTH + 1 + WIDGETS },
  nuxt: {
    fixed: KIT.length + GLOBALS.length + 1 + NUXT_LAYOUTS.length + 1,
    feature: LAYER_DEPTH + 2 + WIDGETS,
  },
};
export const minSfcs = (kind: ScaleKind): number => SHAPE[kind].fixed + SHAPE[kind].feature;
export const PLANT_STRIDE = 10;

export function projectShape(
  sfcs: number,
  kind: ScaleKind = "vue",
): { features: number; extraWidgets: number } {
  const { fixed, feature } = SHAPE[kind];
  if (!Number.isInteger(sfcs) || sfcs < fixed + feature) {
    throw new Error(
      `a ${kind} scale project needs an integer of at least ${fixed + feature} SFCs, got ${sfcs}`,
    );
  }
  const features = Math.floor((sfcs - fixed) / feature);
  return { features, extraWidgets: sfcs - fixed - features * feature };
}

export const isPlanted = (k: number, features: number, seed = 0): boolean =>
  (k + seed) % PLANT_STRIDE === 0 || k === features - 1;

export const COMPILER_OPTIONS = {
  target: "ES2022",
  module: "ESNext",
  moduleResolution: "Bundler",
  lib: ["ES2022", "DOM", "DOM.Iterable"],
  strict: true,
  jsx: "preserve",
  skipLibCheck: true,
  isolatedModules: true,
  types: [],
};
const VUE_COMPILER_OPTIONS = { strictTemplates: true };
const INCLUDE = ["src/**/*.ts", "src/**/*.vue"];

const json = (value: unknown) => `${JSON.stringify(value, null, 2)}\n`;

function specifier(from: string, to: string): string {
  const relative = posix.relative(posix.dirname(from), to).replace(/\.ts$/u, "");
  return relative.startsWith(".") ? relative : `./${relative}`;
}

type Built = Omit<ScaleProject, "packages" | "tsconfig">;

function generate(
  options: Required<Omit<ScaleProjectOptions, "dir" | "featurePackages">>,
  place: (logical: string) => string,
): Built {
  const { sfcs, kind, seed } = options;
  const nuxt = kind === "nuxt";
  const { features, extraWidgets } = projectShape(sfcs, kind);
  const files = new Map<string, string>();
  const plants: Plant[] = [];
  const add = (path: string, make: (ctx: Ctx) => Source) => {
    const to = (logical: string) => specifier(path, place(logical));
    const built = make({
      to,
      nuxt,
      seed,
      component: (logical, local) =>
        nuxt
          ? { tag: autoImportName(place(logical)), imports: [] }
          : { tag: local, imports: [`import ${local} from "${to(logical)}";`] },
    });
    if (files.has(path)) throw new Error(`scale project: ${path} generated twice`);
    files.set(path, `${built.lines.join("\n")}\n`);
    for (const { kind: plant, line } of built.plants) {
      plants.push({ kind: plant, file: path, line, codes: PLANT_CODES[plant] });
    }
  };
  const at = (logical: string, make: (ctx: Ctx) => Source) => add(place(logical), make);
  at("shared/types.ts", sharedTypes);
  for (const name of KIT) at(`kit/${name}.vue`, (ctx) => kitComponent(name, ctx.to));
  for (const name of GLOBALS) at(`global/${name}.vue`, (ctx) => globalComponent(name, ctx.to));
  at("App.vue", nuxt ? nuxtApp : app);
  if (nuxt) {
    add("app/pages/index.vue", indexPage);
    for (const name of NUXT_LAYOUTS) add(`app/layouts/${name}.vue`, () => layout(name));
  }
  for (let k = 0; k < features; k++) {
    const dir = featureDir(k);
    const planted = options.plants && isPlanted(k, features, seed);
    at(`${dir}/model.ts`, (ctx) => featureModel(k, ctx.to));
    for (let index = 0; index < LAYER_DEPTH; index++) {
      at(`${dir}/${layerName(index)}.vue`, (ctx) => layer(k, index, planted, ctx));
    }
    at(`${dir}/Page.vue`, (ctx) => page(k, features, planted, ctx));
    const widgets = WIDGETS + (k === features - 1 ? extraWidgets : 0);
    for (let index = 1; index <= widgets; index++) {
      at(`${dir}/${widgetName(index)}.vue`, (ctx) =>
        widget(k, index, index === widgets, planted && index === 1, ctx),
      );
    }
    if (nuxt) {
      const name = dir.slice("features/".length);
      add(`app/composables/useFeature${k}.ts`, (ctx) =>
        composable(k, rowCount(k, seed), ctx.to(`${dir}/model.ts`)),
      );
      add(`app/pages/features/${name}.vue`, () => routePage(k));
    }
  }
  const vue = [...files.keys()].filter((path) => path.endsWith(".vue")).length;
  if (vue !== sfcs) throw new Error(`scale project: generated ${vue} SFCs, asked for ${sfcs}`);
  return { kind, sfcs, features, seed, files, plants };
}

function vueProject(
  options: Required<Omit<ScaleProjectOptions, "dir" | "featurePackages">>,
): ScaleProject {
  const corpus = generate(options, (logical) => `src/${logical}`);
  corpus.files.set(
    "src/components.d.ts",
    `${globalDeclarations((logical) => specifier("src/components.d.ts", `src/${logical}`)).lines.join("\n")}\n`,
  );
  corpus.files.set(
    "tsconfig.json",
    json({
      compilerOptions: { ...COMPILER_OPTIONS, noEmit: true },
      vueCompilerOptions: VUE_COMPILER_OPTIONS,
      include: INCLUDE,
    }),
  );
  return { ...corpus, packages: [], tsconfig: "tsconfig.json" };
}

function nuxtProject(
  options: Required<Omit<ScaleProjectOptions, "dir" | "featurePackages">>,
): ScaleProject {
  const corpus = generate(options, nuxtPlace);
  corpus.files.set("nuxt.config.ts", NUXT_CONFIG);
  corpus.files.set("tsconfig.json", json(NUXT_TSCONFIG));
  corpus.files.set("package.json", json({ name: "scale-nuxt-app", private: true, type: "module" }));
  return { ...corpus, packages: [], tsconfig: NUXT_APP_TSCONFIG };
}

/** Output directory of one checker variant: one segment, so emitted relative imports keep their depth. */
export const monorepoOutDir = (variant: string) => `.out-${variant}`;

/**
 * `featurePackages` composite packages plus `kit` (shared types, kit and
 * global components) and `app`. Pages import child pages across packages by
 * relative path; each package references its transitive dependencies.
 */
function monorepoProject(
  options: Required<Omit<ScaleProjectOptions, "dir" | "featurePackages">>,
  featurePackages: number,
): ScaleProject {
  const { features } = projectShape(options.sfcs, "monorepo");
  if (!Number.isInteger(featurePackages) || featurePackages < 1 || featurePackages > features) {
    throw new Error(`monorepo needs 1..${features} feature packages, got ${featurePackages}`);
  }
  const featurePackage = (k: number) =>
    `packages/feat-${Math.floor((k * featurePackages) / features)}`;
  const place = (logical: string) => {
    const feature = /^features\/f(\d+)\//u.exec(logical);
    if (feature) return `${featurePackage(Number(feature[1]))}/src/${logical}`;
    return logical === "App.vue" ? "packages/app/src/App.vue" : `packages/kit/src/${logical}`;
  };
  const corpus = generate(options, place);
  const featureNames = Array.from({ length: featurePackages }, (_, p) => `packages/feat-${p}`);
  const direct = new Map<string, Set<string>>([
    ["packages/kit", new Set()],
    ...featureNames.map((name) => [name, new Set(["packages/kit"])] as const),
    ["packages/app", new Set([featurePackage(0)])],
  ]);
  for (let k = 0; k < features; k++) {
    for (const child of childPages(k, features)) {
      if (featurePackage(child) !== featurePackage(k)) {
        direct.get(featurePackage(k))?.add(featurePackage(child));
      }
    }
  }
  const closure = (name: string, seen = new Set<string>()): Set<string> => {
    for (const dep of direct.get(name) ?? []) {
      if (!seen.has(dep)) {
        seen.add(dep);
        closure(dep, seen);
      }
    }
    return seen;
  };
  const packages = ["packages/kit", ...[...featureNames].reverse(), "packages/app"];
  for (const name of featureNames) {
    const path = `${name}/src/components.d.ts`;
    corpus.files.set(
      path,
      `${globalDeclarations((logical) => specifier(path, place(logical))).lines.join("\n")}\n`,
    );
  }
  corpus.files.set(
    "tsconfig.base.json",
    json({
      compilerOptions: { ...COMPILER_OPTIONS, composite: true, emitDeclarationOnly: true },
      vueCompilerOptions: VUE_COMPILER_OPTIONS,
    }),
  );
  for (const name of packages) {
    corpus.files.set(
      `${name}/tsconfig.json`,
      json({
        extends: "../../tsconfig.base.json",
        compilerOptions: {
          rootDir: "src",
          outDir: monorepoOutDir("tsc"),
          tsBuildInfoFile: `${monorepoOutDir("tsc")}/tsconfig.tsbuildinfo`,
        },
        include: INCLUDE,
        references: [...closure(name)].sort().map((dep) => ({ path: posix.relative(name, dep) })),
      }),
    );
  }
  corpus.files.set(
    "tsconfig.json",
    json({ files: [], references: packages.map((name) => ({ path: `./${name}` })) }),
  );
  return { ...corpus, packages, tsconfig: "tsconfig.json" };
}

export function generateScaleProject(options: ScaleProjectOptions): ScaleProject {
  const base = {
    sfcs: options.sfcs,
    kind: options.kind,
    seed: options.seed ?? 0,
    plants: options.plants ?? true,
  };
  if (!Number.isInteger(base.seed) || base.seed < 0)
    throw new Error(`seed must be a natural number, got ${base.seed}`);
  const project =
    options.kind === "monorepo"
      ? monorepoProject(base, options.featurePackages ?? 6)
      : options.kind === "nuxt"
        ? nuxtProject(base)
        : vueProject(base);
  if (options.dir) writeProject(options.dir, project.files);
  return project;
}

/**
 * Per-package and root mapper configs for a monorepo: `references` are not
 * inherited through `extends`, so each config repeats them with the mapper's
 * file name and its own output directory.
 */
export function monorepoMapperConfigs(
  project: ScaleProject,
  mapper: { id: string; package: string },
  configName: string,
): Map<string, string> {
  const out = new Map<string, string>();
  const outDir = monorepoOutDir(`${mapper.id}-mapper`);
  for (const name of project.packages) {
    const base = JSON.parse(project.files.get(`${name}/tsconfig.json`) ?? "null") as {
      references: { path: string }[];
    } | null;
    if (!base) throw new Error(`monorepo package ${name} has no tsconfig.json`);
    out.set(
      `${name}/${configName}`,
      json({
        extends: "./tsconfig.json",
        compilerOptions: { outDir, tsBuildInfoFile: `${outDir}/tsconfig.tsbuildinfo` },
        contentMappers: [{ package: mapper.package, extensions: [".vue"] }],
        references: base.references.map(({ path }) => ({ path: `${path}/${configName}` })),
      }),
    );
  }
  out.set(
    configName,
    json({
      files: [],
      references: project.packages.map((name) => ({ path: `./${name}/${configName}` })),
    }),
  );
  return out;
}

export function writeProject(dir: string, files: Map<string, string>, clean = true): void {
  if (clean) rmSync(dir, { recursive: true, force: true });
  const made = new Set<string>();
  for (const [path, text] of files) {
    const target = join(dir, ...path.split("/"));
    const parent = dirname(target);
    if (!made.has(parent)) {
      mkdirSync(parent, { recursive: true });
      made.add(parent);
    }
    writeFileSync(target, text);
  }
}

/** Plant counts by kind, for summaries. */
export function plantCounts(plants: readonly Plant[]): Record<string, number> {
  const counts: Record<string, number> = {};
  for (const plant of plants) counts[plant.kind] = (counts[plant.kind] ?? 0) + 1;
  return counts;
}
