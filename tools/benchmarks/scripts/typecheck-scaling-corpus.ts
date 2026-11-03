/**
 * Generated Vue projects for the type-check scaling lane: a single project of
 * N SFCs, or the same features split into a TypeScript project-references
 * monorepo. Work per SFC is fixed by construction (fixed layer depth, widget
 * chain and component kit), so total cost should grow linearly with N.
 *
 * Each feature is a page, a 10-deep chain of generic layers forwarding a
 * scoped slot, kebab-case emits and `v-model:query`, and a 9-deep widget
 * chain. Pages form a binary tree, so the deepest import paths grow with
 * log N. Every PLANT_STRIDE-th feature and the last one carry one type error
 * of each PlantKind at a known line.
 */

import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, posix } from "node:path";

import { app, childPages, featureDir, page, widget } from "./typecheck-scaling-feature.ts";
import {
  GLOBALS,
  KIT,
  LAYER_DEPTH,
  type PlantKind,
  type Source,
  type Spec,
  WIDGETS,
  featureModel,
  globalComponent,
  globalDeclarations,
  kitComponent,
  layer,
  layerName,
  sharedTypes,
  widgetName,
} from "./typecheck-scaling-sfc.ts";

export type Plant = { kind: PlantKind; file: string; line: number };
export type Corpus = {
  kind: "project" | "monorepo";
  sfcs: number;
  features: number;
  /** Project-relative posix path to file text. */
  files: Map<string, string>;
  plants: Plant[];
  /** Monorepo package directories, in build order; empty for a single project. */
  packages: string[];
};

export const FIXED_SFCS = KIT.length + GLOBALS.length + 1;
export const FEATURE_SFCS = LAYER_DEPTH + 1 + WIDGETS;
export const MIN_SFCS = FIXED_SFCS + FEATURE_SFCS;
export const PLANT_STRIDE = 10;

export function corpusShape(sfcs: number): { features: number; extraWidgets: number } {
  if (!Number.isInteger(sfcs) || sfcs < MIN_SFCS) {
    throw new Error(`scaling corpus needs an integer of at least ${MIN_SFCS} SFCs, got ${sfcs}`);
  }
  const features = Math.floor((sfcs - FIXED_SFCS) / FEATURE_SFCS);
  return { features, extraWidgets: sfcs - FIXED_SFCS - features * FEATURE_SFCS };
}

export const isPlanted = (k: number, features: number): boolean =>
  k % PLANT_STRIDE === 0 || k === features - 1;

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

type Layout = { place: (logical: string) => string; feature: (k: number) => string };

function generate(sfcs: number, layout: Layout): Omit<Corpus, "kind" | "packages"> {
  const { features, extraWidgets } = corpusShape(sfcs);
  const files = new Map<string, string>();
  const plants: Plant[] = [];
  const add = (path: string, make: (to: Spec) => Source) => {
    const built = make((logical) => specifier(path, layout.place(logical)));
    if (files.has(path)) throw new Error(`scaling corpus: ${path} generated twice`);
    files.set(path, `${built.lines.join("\n")}\n`);
    for (const { kind, line } of built.plants) plants.push({ kind, file: path, line });
  };
  const at = (logical: string, make: (to: Spec) => Source) => add(layout.place(logical), make);
  at("shared/types.ts", sharedTypes);
  for (const name of KIT) at(`kit/${name}.vue`, (to) => kitComponent(name, to));
  for (const name of GLOBALS) at(`global/${name}.vue`, (to) => globalComponent(name, to));
  at("App.vue", app);
  for (let k = 0; k < features; k++) {
    const dir = featureDir(k);
    const planted = isPlanted(k, features);
    at(`${dir}/model.ts`, (to) => featureModel(k, to));
    for (let index = 0; index < LAYER_DEPTH; index++) {
      at(`${dir}/${layerName(index)}.vue`, (to) => layer(k, index, planted, to));
    }
    at(`${dir}/Page.vue`, (to) => page(k, features, planted, to));
    const widgets = WIDGETS + (k === features - 1 ? extraWidgets : 0);
    for (let index = 1; index <= widgets; index++) {
      at(`${dir}/${widgetName(index)}.vue`, (to) =>
        widget(k, index, index === widgets, planted && index === 1, to),
      );
    }
  }
  const vue = [...files.keys()].filter((path) => path.endsWith(".vue")).length;
  if (vue !== sfcs) throw new Error(`scaling corpus: generated ${vue} SFCs, asked for ${sfcs}`);
  return { sfcs, features, files, plants };
}

export function generateProject(sfcs: number): Corpus {
  const corpus = generate(sfcs, {
    place: (logical) => `src/${logical}`,
    feature: () => "src",
  });
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
  return { kind: "project", packages: [], ...corpus };
}

/** Output directory of one checker variant: one segment, so emitted relative imports keep their depth. */
export const monorepoOutDir = (variant: string) => `.out-${variant}`;

/**
 * The same features as `generateProject`, in `featurePackages` composite
 * packages plus `kit` (shared types, kit and global components) and `app`.
 * Pages import child pages across packages by relative path; each package
 * references its transitive dependencies and has its own `components.d.ts`.
 */
export function generateMonorepo(sfcs: number, featurePackages: number): Corpus {
  const { features } = corpusShape(sfcs);
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
  const corpus = generate(sfcs, { place, feature: featurePackage });
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
  // Dependencies first, so a solution file lists packages in build order.
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
  return { ...corpus, kind: "monorepo", packages };
}

/**
 * Per-package and root mapper configs for a monorepo: `references` are not
 * inherited through `extends`, so each config repeats them with the mapper's
 * file name and its own output directory.
 */
export function monorepoMapperConfigs(
  corpus: Corpus,
  mapper: { id: string; package: string },
  configName: string,
): Map<string, string> {
  const out = new Map<string, string>();
  const outDir = monorepoOutDir(`${mapper.id}-mapper`);
  for (const name of corpus.packages) {
    const base = JSON.parse(corpus.files.get(`${name}/tsconfig.json`) ?? "null") as {
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
      references: corpus.packages.map((name) => ({ path: `./${name}/${configName}` })),
    }),
  );
  return out;
}

export function writeCorpus(dir: string, files: Map<string, string>, clean = true): void {
  if (clean) rmSync(dir, { recursive: true, force: true });
  for (const [path, text] of files) {
    const target = join(dir, ...path.split("/"));
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, text);
  }
}

/** Plant counts by kind, for the summary. */
export function plantCounts(plants: Plant[]): Record<string, number> {
  const counts: Record<string, number> = {};
  for (const plant of plants) counts[plant.kind] = (counts[plant.kind] ?? 0) + 1;
  return counts;
}
