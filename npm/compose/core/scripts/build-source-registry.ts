/**
 * Emit `registry/registry.json` and `registry/files/**` for `@vizejs/composable`.
 *
 * Runs as part of `pnpm build` so the published tarball carries the source
 * registry consumed by `vize lib`. Items project `COMPOSABLE_CATALOG.entries`;
 * the metadata-only `./catalog` entry is not a pullable unit. The shared,
 * dependency-free builder lives beside the `@vizejs/ui` scripts so both
 * packages publish byte-identical registry shapes.
 */
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import type { LibRegistrySeedItem } from "../../../ui/scripts/source-registry-bundle/types.ts";
import {
  emitPackageLibRegistry,
  type PackageLibRegistryOptions,
} from "../../../ui/scripts/source-registry-bundle/write.ts";
import { COMPOSABLE_CATALOG, type ComposableEntryMetadata } from "../src/catalog.ts";

/** Default consumer directory for pulled composable sources. */
export const COMPOSABLE_LIB_DEFAULT_TARGET_DIRECTORY = "src/composables/vize";

/** Entries that describe the package instead of shipping runtime behavior. */
export const COMPOSABLE_LIB_EXCLUDED_SUBPATHS: ReadonlySet<string> = new Set(["./catalog"]);

const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function describeEntry(entry: ComposableEntryMetadata, names: readonly string[]): string {
  const categories = [
    ...new Set(
      COMPOSABLE_CATALOG.utilities
        .filter((utility) => utility.entry === entry.subpath)
        .map((utility) => utility.category),
    ),
  ];
  const exported = names.length === 0 ? entry.runtimeExports : names;
  const suffix = categories.length === 0 ? "" : ` (${categories.join(", ")})`;
  return exported.length === 0
    ? `Composable module ${entry.subpath.slice(2)}${suffix}.`
    : `Provides ${exported.join(", ")}${suffix}.`;
}

/** Project one catalog entry into a registry seed item. */
export function composableSeedItem(entry: ComposableEntryMetadata): LibRegistrySeedItem {
  const name = entry.subpath.slice(2);
  const source = entry.source.slice("src/".length);
  return {
    name,
    title: name,
    description: describeEntry(entry, entry.utilities),
    aliases: [...new Set([...entry.utilities, ...entry.runtimeExports])].filter(
      (alias) => alias !== name,
    ),
    packageSubpath: entry.subpath,
    entry: source,
    files: [source],
  };
}

/** Registry options for the checked-out `@vizejs/composable` package. */
export function composableLibRegistryOptions(): PackageLibRegistryOptions {
  return {
    packageRoot,
    kind: "composable",
    defaultTargetDirectory: COMPOSABLE_LIB_DEFAULT_TARGET_DIRECTORY,
    items: COMPOSABLE_CATALOG.entries
      .filter((entry) => !COMPOSABLE_LIB_EXCLUDED_SUBPATHS.has(entry.subpath))
      .map(composableSeedItem),
  };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  const manifestPath = emitPackageLibRegistry(composableLibRegistryOptions());
  process.stdout.write(`wrote ${path.relative(process.cwd(), manifestPath)}\n`);
}
