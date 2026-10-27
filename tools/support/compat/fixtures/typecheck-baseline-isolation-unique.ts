import {
  existsSync,
  lstatSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  realpathSync,
  symlinkSync,
  type Dirent,
} from "node:fs";
import { dirname, join, relative, resolve } from "node:path";

import { readDeclaredPackagePaths, type ShadowedPackage } from "./typecheck-baseline-isolation.ts";
import { ancestorPackagePath } from "./typecheck-baseline-isolation-package-extends.ts";

/**
 * Close the remaining type-reference escape when Nuxt (or another generator)
 * bakes `compilerOptions.paths` to a package *outside* the fixture (#4461).
 *
 * `typecheck-baseline-isolation.ts` will not materialize an outside target —
 * that would import the contamination. If the fixture's own pnpm store holds
 * exactly one copy of that name, this links that copy instead of walking out to
 * Vize's. Several copies are different Vue peer suffixes: this then picks the
 * copy whose pnpm id matches the fixture's own `vue` version, and still does
 * not guess when zero or several copies match.
 *
 * Declared names come from the same `extends` / `references` walk isolation
 * uses, so a check tsconfig that only extends the generated app config still
 * sees the outside mapping (reka-ui's `tsconfig.check.json`). Package-name
 * `extends` specifiers, `compilerOptions.types` entries, plugin packages, and
 * `jsxImportSource` are included as ancestor targets so this can link the
 * fixture's own `@vue/tsconfig`, `vite`, `@types/node`, language plugin, or
 * `vue` JSX runtime before TypeScript climbs into Vize.
 *
 * Undeclared Vue compiler packages (`@vue/compiler-sfc` and friends) and
 * class-component helpers live in Vize's `tests/package.json`, so TypeScript
 * can still climb into them and load Vize's Vue beside the fixture.
 * `@vue/runtime-dom` and `@vue/runtime-core` are nested under pnpm's `vue`
 * rather than hoisted, so ancestor walks miss them; a fixture store copy is
 * still linked so overlay paths can pin the vue-tsc program onto one Vue.
 */

const vueRuntimePackages = [
  "@vue/compiler-core",
  "@vue/compiler-dom",
  "@vue/compiler-sfc",
  "@vue/runtime-core",
  "@vue/runtime-dom",
  "vue",
  "vue-class-component",
  "vue-property-decorator",
  "vue-router",
];

const vueRuntimeNestedPackages = ["@vue/runtime-core", "@vue/runtime-dom"];

export function isolateUniqueLocalTypePackages(
  fixtureRoot: string,
  sourceConfigPath: string,
): ShadowedPackage[] {
  const root = resolve(fixtureRoot);
  return isolateUniqueDeclaredLocalTypePackages(
    root,
    readDeclaredPackagePaths(root, sourceConfigPath),
  );
}

export function isolateUniqueVueRuntimePackages(fixtureRoot: string): ShadowedPackage[] {
  return [
    ...isolateUniqueNamedLocalTypePackages(fixtureRoot, vueRuntimePackages),
    ...isolateUniqueStoreLocalTypePackages(fixtureRoot, vueRuntimeNestedPackages),
    ...isolateUniqueVueVisualizationPackages(fixtureRoot),
  ];
}

export function isolateUniqueVueI18nPackages(fixtureRoot: string): ShadowedPackage[] {
  return isolateUniqueNamedLocalTypePackages(fixtureRoot, ["vue-i18n"]);
}

function isolateUniqueNamedLocalTypePackages(
  fixtureRoot: string,
  names: readonly string[],
): ShadowedPackage[] {
  const root = resolve(fixtureRoot);
  const declared = new Map<string, string>();
  for (const name of names) {
    const ancestor = ancestorPackagePath(root, name);
    if (ancestor == null) continue;
    declared.set(name, ancestor);
  }
  return isolateUniqueDeclaredLocalTypePackages(root, declared);
}

function isolateUniqueStoreLocalTypePackages(
  fixtureRoot: string,
  names: readonly string[],
): ShadowedPackage[] {
  const root = resolve(fixtureRoot);
  const shadowed: ShadowedPackage[] = [];
  for (const name of names) {
    const link = join(root, "node_modules", ...name.split("/"));
    if (existsSync(link) || isDanglingLink(link)) continue;
    const copies = findLocalCopies(root, name);
    const local = selectLocalCopy(root, name, copies);
    if (local == null) continue;
    mkdirSync(dirname(link), { recursive: true });
    symlinkSync(relative(dirname(link), local), link);
    shadowed.push({ name, target: relative(root, local).replaceAll("\\", "/") });
  }
  return shadowed.sort((left, right) => compare(left.name, right.name));
}

function isolateUniqueDeclaredLocalTypePackages(
  root: string,
  declared: ReadonlyMap<string, string>,
): ShadowedPackage[] {
  if (declared.size === 0) return [];
  const reachable = collectAncestorPackageNames(root);
  const shadowed: ShadowedPackage[] = [];
  for (const [name, target] of [...declared].sort(([left], [right]) => compare(left, right))) {
    if (!reachable.has(name)) continue;
    const link = join(root, "node_modules", name);
    if (existsSync(link) || isDanglingLink(link)) continue;
    if (isPackageDirectory(target) && isInside(root, target)) continue;
    const copies = findLocalCopies(root, name);
    const local = selectLocalCopy(root, name, copies);
    if (local == null) continue;
    mkdirSync(dirname(link), { recursive: true });
    symlinkSync(relative(dirname(link), local), link);
    shadowed.push({ name, target: relative(root, local).replaceAll("\\", "/") });
  }
  return shadowed;
}

function findLocalCopies(fixtureRoot: string, name: string): string[] {
  const copies = new Map<string, string>();
  const store = join(fixtureRoot, "node_modules", ".pnpm");
  const segments = name.split("/");
  for (const entry of readDirectory(store)) {
    if (entry.name.startsWith(".")) continue;
    const packageDir = join(store, entry.name, "node_modules", ...segments);
    if (!isPackageDirectory(packageDir) || !isInside(fixtureRoot, packageDir)) continue;
    let real: string;
    try {
      real = realpathSync(packageDir);
    } catch {
      continue;
    }
    if (!isInside(fixtureRoot, real)) continue;
    copies.set(real, packageDir);
  }
  return [...copies.values()].sort(compare);
}

function selectLocalCopy(
  fixtureRoot: string,
  name: string,
  copies: readonly string[],
): string | null {
  if (copies.length === 1) return copies[0];
  if (copies.length === 0) return null;
  const vueVersion = readFixtureVueVersion(fixtureRoot);
  if (vueVersion == null) return null;
  const matched = copies.filter((copy) => copyMatchesVue(fixtureRoot, name, copy, vueVersion));
  return matched.length === 1 ? matched[0] : null;
}

function readFixtureVueVersion(fixtureRoot: string): string | null {
  const hoisted = readPackageVersion(join(fixtureRoot, "node_modules", "vue"));
  if (hoisted != null) return hoisted;
  const copies = findLocalCopies(fixtureRoot, "vue");
  if (copies.length !== 1) return null;
  return readPackageVersion(copies[0]);
}

function readPackageVersion(packageDir: string): string | null {
  try {
    const pkg = JSON.parse(readFileSync(join(packageDir, "package.json"), "utf8")) as {
      readonly version?: unknown;
    };
    return typeof pkg.version === "string" && pkg.version !== "" ? pkg.version : null;
  } catch {
    return null;
  }
}

function copyMatchesVue(
  fixtureRoot: string,
  name: string,
  packageDir: string,
  vueVersion: string,
): boolean {
  const store = join(fixtureRoot, "node_modules", ".pnpm");
  const relativePath = relative(store, packageDir);
  if (relativePath.startsWith("..") || relativePath.startsWith("/")) return false;
  const id = relativePath.split(/[\\/]/)[0];
  const marker = name === "vue" ? `vue@${vueVersion}` : `_vue@${vueVersion}`;
  const index = id.indexOf(marker);
  if (index === -1) return false;
  if (name === "vue" && index !== 0) return false;
  const end = index + marker.length;
  return end === id.length || id[end] === "_" || id[end] === "(";
}

function collectAncestorPackageNames(fixtureRoot: string): Set<string> {
  const names = new Set<string>();
  let directory = dirname(fixtureRoot);
  let previous: string | null = null;
  while (directory !== previous) {
    collectPackageNames(join(directory, "node_modules"), names);
    previous = directory;
    directory = dirname(directory);
  }
  return names;
}

function collectPackageNames(nodeModules: string, names: Set<string>): void {
  for (const entry of readDirectory(nodeModules)) {
    if (entry.name.startsWith(".")) continue;
    if (!entry.name.startsWith("@")) {
      names.add(entry.name);
      continue;
    }
    for (const scoped of readDirectory(join(nodeModules, entry.name))) {
      names.add(`${entry.name}/${scoped.name}`);
    }
  }
}

function readDirectory(directory: string): Dirent[] {
  try {
    return readdirSync(directory, { withFileTypes: true });
  } catch {
    return [];
  }
}

function isPackageDirectory(target: string): boolean {
  return existsSync(join(target, "package.json"));
}

function isInside(root: string, target: string): boolean {
  const path = relative(root, target);
  return path !== "" && !path.startsWith("..") && !path.startsWith("/");
}

function isDanglingLink(link: string): boolean {
  try {
    lstatSync(link);
    return true;
  } catch {
    return false;
  }
}

function compare(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

export function isolateUniqueVueUsePackages(fixtureRoot: string): ShadowedPackage[] {
  return isolateUniqueNamedLocalTypePackages(fixtureRoot, ["@vueuse/core", "@vueuse/shared"]);
}

export function isolateUniqueNuxtUiPackages(fixtureRoot: string): ShadowedPackage[] {
  return isolateUniqueNamedLocalTypePackages(fixtureRoot, ["@nuxt/ui"]);
}

export function isolateUniqueUiLibraryPackages(fixtureRoot: string): ShadowedPackage[] {
  return isolateUniqueNamedLocalTypePackages(fixtureRoot, [
    "@ionic/vue",
    "@nuxt/ui",
    "ant-design-vue",
    "element-plus",
    "naive-ui",
    "primevue",
    "quasar",
    "reka-ui",
    "swiper",
    "vant",
    "vue-select",
    "vue-virtual-scroller",
  ]);
}

export function isolateUniqueVueFormPackages(fixtureRoot: string): ShadowedPackage[] {
  return isolateUniqueNamedLocalTypePackages(fixtureRoot, ["@formkit/vue", "vee-validate"]);
}

export function isolateUniqueVueQueryPackages(fixtureRoot: string): ShadowedPackage[] {
  return isolateUniqueNamedLocalTypePackages(fixtureRoot, [
    "@tanstack/vue-query",
    "@vue/apollo-composable",
  ]);
}

export function isolateUniqueVueEditorPackages(fixtureRoot: string): ShadowedPackage[] {
  return isolateUniqueNamedLocalTypePackages(fixtureRoot, ["@tiptap/vue-3", "@vue-flow/core"]);
}

export function isolateUniqueVueVisualizationPackages(fixtureRoot: string): ShadowedPackage[] {
  return isolateUniqueNamedLocalTypePackages(fixtureRoot, ["@tresjs/core", "vue-chartjs"]);
}
