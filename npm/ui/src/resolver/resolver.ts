/**
 * Build-time resolvers for `unplugin-vue-components`, `unplugin-auto-import`,
 * and framework modules (Nuxt). Everything resolves to direct family subpath
 * imports (`@vizejs/ui/<family>`), so auto-imported code tree-shakes exactly
 * like hand-written imports and nothing here ships to the browser.
 *
 * With `source: "local"`, families pulled into the project with
 * `vize lib pull` (recorded in `vize-lib.lock.json`) resolve to the local
 * copies instead of the package.
 */
import { readFileSync } from "node:fs";
import path from "node:path";

import {
  UI_RESOLVER_COMPONENTS,
  UI_RESOLVER_COMPOSABLES,
  UI_RESOLVER_ENTRIES,
} from "./resolver-manifest.ts";

/** Every auto-registrable component export name. */
export type VizeUiComponentName = keyof typeof UI_RESOLVER_COMPONENTS;

/** Every auto-importable `use*` export name. */
export type VizeUiComposableName = keyof typeof UI_RESOLVER_COMPOSABLES;

/** Family subpath (without `./`) that owns at least one resolvable export. */
export type VizeUiFamily = keyof typeof UI_RESOLVER_ENTRIES;

/** One `vize-lib.lock.json` item (the fields resolvers read). */
export interface VizeLibLockedItem {
  /** Item name, for example `"button"` or `"use-toggle"`. */
  readonly name: string;
  /** Item kind, for example `"ui"` or `"composable"`. */
  readonly kind: string;
  /** Project-relative POSIX directory the registry layout was copied into. */
  readonly dir: string;
}

/** The parts of `vize-lib.lock.json` resolvers read. */
export interface VizeLibLockfile {
  /** Lockfile schema version. */
  readonly lockfileVersion: number;
  /** Pulled items. */
  readonly items: readonly VizeLibLockedItem[];
}

/** Options shared by the `@vizejs/ui` resolvers. */
export interface VizeUiResolverOptions {
  /**
   * Prefix for component names in templates, for example `"Vz"` for `<VzButton>`.
   *
   * @default ""
   */
  readonly prefix?: string;

  /**
   * Only resolve these families.
   *
   * @default every family
   */
  readonly include?: readonly VizeUiFamily[];

  /**
   * Never resolve these families.
   *
   * @default []
   */
  readonly exclude?: readonly VizeUiFamily[];

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
  readonly lockfile?: string | VizeLibLockfile;

  /**
   * In `local` mode, resolve families that were not pulled to the package.
   *
   * @default true
   */
  readonly fallback?: boolean;
}

/** An import produced by a resolver: `import { name as as } from from`. */
export interface VizeResolvedImport {
  /** Export name in the source module. */
  readonly name: string;
  /** Local binding name when it differs from `name` (for prefixed components). */
  readonly as?: string;
  /** Module specifier or absolute file path. */
  readonly from: string;
}

/** `unplugin-vue-components` component resolver object. */
export interface VizeComponentResolver {
  /** Resolver kind. */
  readonly type: "component";
  /** Resolve a template tag name. */
  readonly resolve: (name: string) => VizeResolvedImport | undefined;
}

/** Package that owns the resolvable exports. */
export const VIZE_UI_PACKAGE = "@vizejs/ui";

const lockfileDiagnostic = "VIZE_UI_RESOLVER_LOCKFILE";

function isLockfile(value: unknown): value is VizeLibLockfile {
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

/** Read and validate `vize-lib.lock.json`. Throws a tagged error when it is missing or malformed. */
export function readVizeLibLockfile(file: string): VizeLibLockfile {
  let parsed: unknown;
  try {
    parsed = JSON.parse(readFileSync(file, "utf8"));
  } catch (error) {
    throw new Error(`${lockfileDiagnostic}: cannot read ${file}`, { cause: error });
  }
  if (!isLockfile(parsed))
    throw new Error(`${lockfileDiagnostic}: ${file} is not a vize lib lockfile`);
  return parsed;
}

function toPosix(value: string): string {
  return value.split(path.sep).join("/");
}

/** Create the family -> module specifier mapping for the given options. */
function createFamilyLocator(
  options: VizeUiResolverOptions,
): (family: VizeUiFamily) => string | undefined {
  const include = options.include === undefined ? undefined : new Set<string>(options.include);
  const exclude = new Set<string>(options.exclude ?? []);
  const allowed = (family: string) =>
    (include === undefined || include.has(family)) && !exclude.has(family);
  if (options.source !== "local") {
    return (family) => (allowed(family) ? `${VIZE_UI_PACKAGE}/${family}` : undefined);
  }
  const root = options.root ?? process.cwd();
  const lockfile =
    typeof options.lockfile === "object"
      ? options.lockfile
      : readVizeLibLockfile(path.resolve(root, options.lockfile ?? "vize-lib.lock.json"));
  const pulled = new Map(
    lockfile.items
      .filter((item) => item.kind === "ui")
      .map((item) => [item.name, item.dir] as const),
  );
  const fallback = options.fallback ?? true;
  return (family) => {
    if (!allowed(family)) return undefined;
    const dir = pulled.get(family);
    if (dir === undefined) return fallback ? `${VIZE_UI_PACKAGE}/${family}` : undefined;
    return toPosix(path.resolve(root, dir, UI_RESOLVER_ENTRIES[family]));
  };
}

function hasKey<Record extends object>(
  record: Record,
  key: string,
): key is Extract<keyof Record, string> {
  return Object.prototype.hasOwnProperty.call(record, key);
}

/** Every component that the options allow, with its resolved import. */
export function listVizeUiComponents(
  options: VizeUiResolverOptions = {},
): readonly VizeResolvedImport[] {
  const locate = createFamilyLocator(options);
  const prefix = options.prefix ?? "";
  const imports: VizeResolvedImport[] = [];
  for (const name of Object.keys(UI_RESOLVER_COMPONENTS)) {
    if (!hasKey(UI_RESOLVER_COMPONENTS, name)) continue;
    const from = locate(UI_RESOLVER_COMPONENTS[name]);
    if (from === undefined) continue;
    imports.push(prefix.length === 0 ? { name, from } : { name, as: `${prefix}${name}`, from });
  }
  return imports;
}

/** Every `use*` composable that the options allow, with its resolved import. */
export function listVizeUiComposables(
  options: VizeUiResolverOptions = {},
): readonly VizeResolvedImport[] {
  const locate = createFamilyLocator(options);
  const imports: VizeResolvedImport[] = [];
  for (const name of Object.keys(UI_RESOLVER_COMPOSABLES)) {
    if (!hasKey(UI_RESOLVER_COMPOSABLES, name)) continue;
    const from = locate(UI_RESOLVER_COMPOSABLES[name]);
    if (from !== undefined) imports.push({ name, from });
  }
  return imports;
}

function pascalCase(tag: string): string {
  return tag
    .split(/[-_]/)
    .filter((part) => part.length > 0)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
}

/**
 * `unplugin-vue-components` resolver for every `@vizejs/ui` component.
 *
 * @example
 * ```ts
 * Components({ resolvers: [VizeUiResolver({ prefix: "Vz" })] });
 * // <VzDialogTrigger> -> import { DialogTrigger as VzDialogTrigger } from "@vizejs/ui/dialog"
 * ```
 */
export function VizeUiResolver(options: VizeUiResolverOptions = {}): VizeComponentResolver {
  const locate = createFamilyLocator(options);
  const prefix = options.prefix ?? "";
  return {
    type: "component",
    resolve(tag: string) {
      const name = pascalCase(tag);
      if (!name.startsWith(prefix)) return undefined;
      const exportName = name.slice(prefix.length);
      if (!hasKey(UI_RESOLVER_COMPONENTS, exportName)) return undefined;
      const from = locate(UI_RESOLVER_COMPONENTS[exportName]);
      if (from === undefined) return undefined;
      return prefix.length === 0
        ? { name: exportName, from }
        : { name: exportName, as: name, from };
    },
  };
}

/**
 * `unplugin-auto-import` resolver for `@vizejs/ui` composables (`useFieldWiring`, …).
 *
 * @example
 * ```ts
 * AutoImport({ resolvers: [VizeUiComposablesResolver()] });
 * ```
 */
export function VizeUiComposablesResolver(
  options: Omit<VizeUiResolverOptions, "prefix"> = {},
): (name: string) => VizeResolvedImport | undefined {
  const locate = createFamilyLocator(options);
  return (name: string) => {
    if (!hasKey(UI_RESOLVER_COMPOSABLES, name)) return undefined;
    const from = locate(UI_RESOLVER_COMPOSABLES[name]);
    return from === undefined ? undefined : { name, from };
  };
}

/** `unplugin-auto-import` `imports` preset: `{ "@vizejs/ui/<family>": ["useX", …] }`. */
export function vizeUiImports(
  options: Omit<VizeUiResolverOptions, "prefix"> = {},
): Readonly<Record<string, readonly string[]>> {
  const grouped: Record<string, string[]> = {};
  for (const { name, from } of listVizeUiComposables(options)) (grouped[from] ??= []).push(name);
  return grouped;
}

/**
 * `GlobalComponents` declarations for every resolvable component, for setups
 * that register components without a generator (for example a Vite plugin
 * of your own). Nuxt and `unplugin-vue-components` generate these themselves.
 */
export function createVizeUiComponentDeclarations(options: VizeUiResolverOptions = {}): string {
  const lines = listVizeUiComponents(options).map(
    (item) =>
      `    ${item.as ?? item.name}: typeof import(${JSON.stringify(item.from)})["${item.name}"];`,
  );
  return [
    "// Generated by @vizejs/ui/resolver. Do not edit.",
    "export {};",
    'declare module "vue" {',
    "  export interface GlobalComponents {",
    ...lines,
    "  }",
    "}",
    "",
  ].join("\n");
}

export {
  UI_RESOLVER_COMPONENTS,
  UI_RESOLVER_COMPOSABLES,
  UI_RESOLVER_ENTRIES,
} from "./resolver-manifest.ts";
