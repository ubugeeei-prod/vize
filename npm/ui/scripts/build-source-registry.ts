/**
 * Emit `registry/registry.json` and `registry/files/**` for `@vizejs/ui`.
 *
 * Runs as part of `pnpm build` so the published tarball always carries the
 * source registry consumed by `vize lib`. The item list is a projection of
 * `uiFamilyCatalog`; test files are never published.
 */
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { uiFamilyCatalog, type UiFamilyCatalogEntry } from "../src/catalog/family-catalog.ts";
import {
  emitPackageLibRegistry,
  type PackageLibRegistryOptions,
} from "./source-registry-bundle/write.ts";
import type { LibRegistrySeedItem } from "./source-registry-bundle/types.ts";

/** Default consumer directory for pulled UI sources. */
export const UI_LIB_DEFAULT_TARGET_DIRECTORY = "src/components/vize";

const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function stripSourcePrefix(file: string): string {
  return file.startsWith("src/") ? file.slice("src/".length) : file;
}

function describeFamily(entry: UiFamilyCatalogEntry): string {
  const coverage = entry.upstreamCoverage.slice(0, 3).join(", ");
  const kind = entry.sourceFiles.some((file) => file.endsWith(".vue")) ? "component" : "foundation";
  return coverage.length === 0
    ? `Headless ${entry.title} ${kind} (${entry.maturity}).`
    : `Headless ${entry.title} ${kind} (${entry.maturity}); covers ${coverage}.`;
}

/** Project one catalog family into a registry seed item. */
export function uiFamilySeedItem(entry: UiFamilyCatalogEntry): LibRegistrySeedItem {
  const subpathName =
    entry.packageSubpath === "." ? entry.canonicalName : entry.packageSubpath.slice(2);
  return {
    name: entry.canonicalName,
    title: entry.title,
    description: describeFamily(entry),
    aliases: [...new Set([...entry.aliases, subpathName])].filter(
      (alias) => alias !== entry.canonicalName,
    ),
    packageSubpath: entry.packageSubpath,
    entry: stripSourcePrefix(entry.entryFile),
    files: entry.sourceFiles.map(stripSourcePrefix),
  };
}

/** Registry options for the checked-out `@vizejs/ui` package. */
export function uiLibRegistryOptions(): PackageLibRegistryOptions {
  return {
    packageRoot,
    kind: "ui",
    defaultTargetDirectory: UI_LIB_DEFAULT_TARGET_DIRECTORY,
    items: uiFamilyCatalog.map(uiFamilySeedItem),
  };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  const manifestPath = emitPackageLibRegistry(uiLibRegistryOptions());
  process.stdout.write(`wrote ${path.relative(process.cwd(), manifestPath)}\n`);
}
