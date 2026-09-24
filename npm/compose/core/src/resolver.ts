/**
 * Build-time auto-import resolver for `@vizejs/composable`, driven by the
 * package catalog so every new entry is picked up automatically.
 *
 * Every export resolves to its own subpath (`@vizejs/composable/use-toggle`),
 * so auto-imported code tree-shakes exactly like hand-written imports; this
 * module runs in Node (Vite/Nuxt config) and never ships to the browser.
 * With `source: "local"`, entries pulled by `vize lib pull` (recorded in
 * `vize-lib.lock.json`) resolve to the local copies.
 */
import { readFileSync } from "node:fs";
import path from "node:path";

import { COMPOSABLE_CATALOG } from "./catalog.ts";

/** Package that owns the resolvable exports. */
export const VIZE_COMPOSABLE_PACKAGE = "@vizejs/composable";

/** Subpaths that are tooling, not auto-importable runtime code. */
const NON_IMPORTABLE_SUBPATHS: ReadonlySet<string> = new Set(["./catalog", "./resolver"]);

/** Entry name (subpath without `./`) that exposes at least one auto-importable export. */
export type VizeComposableEntry = string;

/** One `vize-lib.lock.json` item (the fields the resolver reads). */
export interface VizeComposableLockedItem {
  /** Item name, for example `"use-toggle"`. */
  readonly name: string;
  /** Item kind, for example `"composable"`. */
  readonly kind: string;
  /** Project-relative POSIX directory the registry layout was copied into. */
  readonly dir: string;
}

/** The parts of `vize-lib.lock.json` the resolver reads. */
export interface VizeComposableLockfile {
  /** Lockfile schema version. */
  readonly lockfileVersion: number;
  /** Pulled items. */
  readonly items: readonly VizeComposableLockedItem[];
}

/** Options for the `@vizejs/composable` resolvers. */
export interface VizeComposableResolverOptions {
  /**
   * Only resolve exports of these entries (for example `"use-toggle"`).
   *
   * @default every entry
   */
  readonly include?: readonly VizeComposableEntry[];

  /**
   * Never resolve exports of these entries.
   *
   * @default []
   */
  readonly exclude?: readonly VizeComposableEntry[];

  /**
   * Only resolve these export names (for example to avoid clashing with another library).
   *
   * @default every runtime export
   */
  readonly names?: readonly string[];

  /**
   * Resolve to the npm package, or to `vize lib pull` copies recorded in the lockfile.
   *
   * @default "package"
   */
  readonly source?: "local" | "package";

  /**
   * Project root used to locate the lockfile and pulled directories.
   *
   * @default process.cwd()
   */
  readonly root?: string;

  /**
   * Lockfile path (relative to `root`) or an already-parsed lockfile.
   *
   * @default "vize-lib.lock.json"
   */
  readonly lockfile?: string | VizeComposableLockfile;

  /**
   * In `local` mode, resolve entries that were not pulled to the package.
   *
   * @default true
   */
  readonly fallback?: boolean;
}

/** An import produced by the resolver: `import { name } from from`. */
export interface VizeComposableImport {
  /** Export name. */
  readonly name: string;
  /** Module specifier or absolute file path. */
  readonly from: string;
}

interface CatalogExport {
  readonly entry: string;
  readonly source: string;
}

/** Export name -> owning entry, first catalog entry wins. */
function catalogExports(): ReadonlyMap<string, CatalogExport> {
  const exports = new Map<string, CatalogExport>();
  for (const entry of COMPOSABLE_CATALOG.entries) {
    if (NON_IMPORTABLE_SUBPATHS.has(entry.subpath)) continue;
    for (const name of entry.runtimeExports) {
      if (!exports.has(name)) {
        exports.set(name, {
          entry: entry.subpath.slice(2),
          source: entry.source.slice("src/".length),
        });
      }
    }
  }
  return exports;
}

function isLockfile(value: unknown): value is VizeComposableLockfile {
  return (
    typeof value === "object" &&
    value !== null &&
    "items" in value &&
    Array.isArray(value.items) &&
    value.items.every(
      (item: unknown) =>
        typeof item === "object" &&
        item !== null &&
        "name" in item &&
        "kind" in item &&
        "dir" in item &&
        typeof item.name === "string" &&
        typeof item.kind === "string" &&
        typeof item.dir === "string",
    )
  );
}

/**
 * Read and validate `vize-lib.lock.json`.
 *
 * @param file Absolute lockfile path.
 * @returns The parsed lockfile.
 * @throws {Error} `[VIZE_COMPOSE_RESOLVER_LOCKFILE]` when missing or malformed.
 */
export function readComposableLockfile(file: string): VizeComposableLockfile {
  let parsed: unknown;
  try {
    parsed = JSON.parse(readFileSync(file, "utf8"));
  } catch (error) {
    throw new Error(`[VIZE_COMPOSE_RESOLVER_LOCKFILE] cannot read ${file}`, { cause: error });
  }
  if (!isLockfile(parsed)) {
    throw new Error(`[VIZE_COMPOSE_RESOLVER_LOCKFILE] ${file} is not a vize lib lockfile`);
  }
  return parsed;
}

function createLocator(
  options: VizeComposableResolverOptions,
): (item: CatalogExport) => string | undefined {
  const include = options.include === undefined ? undefined : new Set(options.include);
  const exclude = new Set(options.exclude ?? []);
  const allowed = (entry: string) =>
    (include === undefined || include.has(entry)) && !exclude.has(entry);
  if (options.source !== "local") {
    return (item) => (allowed(item.entry) ? `${VIZE_COMPOSABLE_PACKAGE}/${item.entry}` : undefined);
  }
  const root = options.root ?? process.cwd();
  const lockfile =
    typeof options.lockfile === "object"
      ? options.lockfile
      : readComposableLockfile(path.resolve(root, options.lockfile ?? "vize-lib.lock.json"));
  const pulled = new Map(
    lockfile.items
      .filter((item) => item.kind === "composable")
      .map((item) => [item.name, item.dir] as const),
  );
  const fallback = options.fallback ?? true;
  return (item) => {
    if (!allowed(item.entry)) return undefined;
    const dir = pulled.get(item.entry);
    if (dir === undefined) return fallback ? `${VIZE_COMPOSABLE_PACKAGE}/${item.entry}` : undefined;
    return path.resolve(root, dir, item.source).split(path.sep).join("/");
  };
}

/**
 * Every auto-importable export the options allow, with its direct subpath import.
 *
 * @param options Filters and source mode.
 * @default options {}
 * @returns Imports in catalog order.
 */
export function listVizeComposableImports(
  options: VizeComposableResolverOptions = {},
): readonly VizeComposableImport[] {
  const locate = createLocator(options);
  const names = options.names === undefined ? undefined : new Set(options.names);
  const imports: VizeComposableImport[] = [];
  for (const [name, item] of catalogExports()) {
    if (names !== undefined && !names.has(name)) continue;
    const from = locate(item);
    if (from !== undefined) imports.push({ name, from });
  }
  return imports;
}

/**
 * `unplugin-auto-import` resolver for every `@vizejs/composable` export.
 *
 * Server/SSR: build-time only; reads the lockfile once in `local` mode.
 *
 * @example
 * ```ts
 * AutoImport({ resolvers: [VizeComposableResolver()] });
 * // useToggle() -> import { useToggle } from "@vizejs/composable/use-toggle"
 * ```
 *
 * @param options Filters and source mode.
 * @default options {}
 * @returns A name -> import resolver function.
 */
export function VizeComposableResolver(
  options: VizeComposableResolverOptions = {},
): (name: string) => VizeComposableImport | undefined {
  const allowed = new Map(listVizeComposableImports(options).map((item) => [item.name, item]));
  return (name: string) => allowed.get(name);
}

/**
 * `unplugin-auto-import` `imports` preset: `{ "@vizejs/composable/use-toggle": ["useToggle"] }`.
 *
 * @param options Filters and source mode.
 * @default options {}
 * @returns Export names grouped by module.
 */
export function vizeComposableImports(
  options: VizeComposableResolverOptions = {},
): Readonly<Record<string, readonly string[]>> {
  const grouped: Record<string, string[]> = {};
  for (const { name, from } of listVizeComposableImports(options))
    (grouped[from] ??= []).push(name);
  return grouped;
}

/**
 * Global declarations for auto-imported composables, for setups without a
 * generator (Nuxt and `unplugin-auto-import` emit their own).
 *
 * @param options Filters and source mode.
 * @default options {}
 * @returns A `.d.ts` module declaring each export as a global.
 */
export function createVizeComposableDeclarations(
  options: VizeComposableResolverOptions = {},
): string {
  const lines = listVizeComposableImports(options).map(
    (item) => `  const ${item.name}: typeof import(${JSON.stringify(item.from)})["${item.name}"];`,
  );
  return [
    "// Generated by @vizejs/composable/resolver. Do not edit.",
    "export {};",
    "declare global {",
    ...lines,
    "}",
    "",
  ].join("\n");
}
