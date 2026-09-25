/**
 * Build the `@vizejs/ui/resolver` manifest from the family catalog.
 *
 * Every catalogued family entry file is parsed for its runtime exports. An
 * export is a component when it resolves (through any chain of named
 * re-exports and `export *`) to the default export of a `.vue` file; an
 * export named `use*` that is not a component is an auto-importable
 * composable. Each name is owned by one family: the family whose own
 * directory contains the resolved source wins over families that merely
 * re-export it, and ties fall back to catalog order.
 */
import { readFileSync } from "node:fs";
import path from "node:path";

import { uiFamilyCatalog } from "../src/catalog/family-catalog.ts";

/** Kind of a resolved runtime export. */
type ExportKind =
  | { readonly kind: "component"; readonly source: string }
  | { readonly kind: "value"; readonly source: string };

/** Generated manifest contents. */
export interface UiResolverManifest {
  /** Component export name -> owning family (package subpath without `./`). */
  readonly components: Readonly<Record<string, string>>;
  /** Composable (`use*`) export name -> owning family. */
  readonly composables: Readonly<Record<string, string>>;
  /** Family -> entry file relative to a pulled registry directory (for `vize lib pull` copies). */
  readonly entries: Readonly<Record<string, string>>;
}

const namedExport = /export\s+(?:type\s+)?\{([^}]*)\}\s*from\s*["']([^"']+)["']/g;
const starExport = /export\s*\*\s*from\s*["']([^"']+)["']/g;
const localExport = /export\s+(?:async\s+)?(?:function\*?|const|let|class)\s+([A-Za-z_$][\w$]*)/g;

function stripComments(source: string): string {
  return source.replace(/\/\*[\s\S]*?\*\//g, "").replace(/^\s*\/\/.*$/gm, "");
}

const resolved = new Map<string, ReadonlyMap<string, ExportKind>>();
const inProgress = new Set<string>();

function resolveExports(file: string): ReadonlyMap<string, ExportKind> {
  const cached = resolved.get(file);
  if (cached !== undefined) return cached;
  const result = new Map<string, ExportKind>();
  // Cycles resolve to what is known so far.
  if (inProgress.has(file)) return result;
  inProgress.add(file);
  try {
    collectExports(file, result);
  } finally {
    inProgress.delete(file);
  }
  resolved.set(file, result);
  return result;
}

function collectExports(file: string, result: Map<string, ExportKind>): void {
  if (file.endsWith(".vue")) {
    result.set("default", { kind: "component", source: file });
    return;
  }
  const source = stripComments(readFileSync(file, "utf8"));
  const dir = path.dirname(file);
  for (const match of source.matchAll(localExport)) {
    const name = match[1];
    if (name !== undefined) result.set(name, { kind: "value", source: file });
  }
  for (const match of source.matchAll(namedExport)) {
    const specifiers = match[1] ?? "";
    const target = path.resolve(dir, match[2] ?? "");
    // `export type { … }` blocks carry no runtime value (the pattern also matches them).
    if (/^export\s+type\b/.test(match[0])) continue;
    let targetExports: ReadonlyMap<string, ExportKind> | undefined;
    for (const raw of specifiers.split(",")) {
      const specifier = raw.trim();
      if (specifier.length === 0 || specifier.startsWith("type ")) continue;
      const [imported = "", exported = imported] = specifier
        .split(/\s+as\s+/)
        .map((part) => part.trim());
      targetExports ??= resolveExports(target);
      result.set(exported, targetExports.get(imported) ?? { kind: "value", source: target });
    }
  }
  for (const match of source.matchAll(starExport)) {
    for (const [name, kind] of resolveExports(path.resolve(dir, match[1] ?? ""))) {
      if (name !== "default" && !result.has(name)) result.set(name, kind);
    }
  }
}

/** Build the manifest from `packageRoot` (the `npm/ui` directory). */
export function buildUiResolverManifest(packageRoot: string): UiResolverManifest {
  const owners = new Map<
    string,
    { family: string; own: boolean; kind: "component" | "composable" }
  >();
  const entries: Record<string, string> = {};
  for (const entry of uiFamilyCatalog) {
    if (entry.packageSubpath === ".") continue;
    const family = entry.packageSubpath.slice(2);
    entries[family] = entry.entryFile.replace(/^src\//, "");
    const familyDir = path.resolve(packageRoot, path.dirname(entry.entryFile));
    const exports = resolveExports(path.resolve(packageRoot, entry.entryFile));
    for (const [name, resolved] of exports) {
      const kind =
        resolved.kind === "component"
          ? "component"
          : /^use[A-Z]/.test(name)
            ? "composable"
            : undefined;
      if (kind === undefined || name === "default") continue;
      const own = resolved.source.startsWith(`${familyDir}${path.sep}`);
      const current = owners.get(name);
      if (current === undefined || (own && !current.own)) owners.set(name, { family, own, kind });
    }
  }
  const components: Record<string, string> = {};
  const composables: Record<string, string> = {};
  for (const name of [...owners.keys()].sort()) {
    const owner = owners.get(name);
    if (owner === undefined) continue;
    if (owner.kind === "component") components[name] = owner.family;
    else composables[name] = owner.family;
  }
  const usedFamilies = new Set([...Object.values(components), ...Object.values(composables)]);
  const usedEntries: Record<string, string> = {};
  for (const family of Object.keys(entries).sort()) {
    const file = entries[family];
    if (usedFamilies.has(family) && file !== undefined) usedEntries[family] = file;
  }
  return { components, composables, entries: usedEntries };
}

/** Render the generated TypeScript module. */
export function renderUiResolverManifest(manifest: UiResolverManifest): string {
  const record = (entries: Readonly<Record<string, string>>) =>
    Object.entries(entries)
      .map(([name, family]) => `  ${name}: "${family}",`)
      .join("\n");
  return `// Generated by scripts/generate-resolver-manifest.ts from the family catalog. Do not edit.

/** Component export name -> \`@vizejs/ui\` family subpath that owns it. */
export const UI_RESOLVER_COMPONENTS = {
${record(manifest.components)}
} as const;

/** Composable (\`use*\`) export name -> \`@vizejs/ui\` family subpath that owns it. */
export const UI_RESOLVER_COMPOSABLES = {
${record(manifest.composables)}
} as const;

/** Family -> entry file inside a \`vize lib pull\` directory. */
export const UI_RESOLVER_ENTRIES = {
${Object.entries(manifest.entries)
  .map(([family, file]) => `  ${JSON.stringify(family)}: "${file}",`)
  .join("\n")}
} as const;
`;
}
