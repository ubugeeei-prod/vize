/**
 * Resolve which component an art file demonstrates, and under what tag.
 *
 * Shared by the art module and by the per-variant SFC compiler (#3857) so both
 * agree on the inferred target import and its template binding name.
 * The parsed `<script setup>` is passed in rather than parsed here, so this
 * module does not have to import back from `art-module.ts`.
 */

import path from "node:path";

import { allowedSourceRoots, resolveComponentSourcePath } from "./component-source.js";
import type { ArtFileInfo } from "./types/index.js";
import { toPascalCase } from "./utils.js";

export type ArtScriptSetupInfo = {
  imports?: string[];
  defineArtComponentName?: string;
  defineArtComponentSource?: string;
  defineArtComponentImport?: string;
} | null;

export type ResolvedArtComponent = {
  componentImportPath?: string;
  componentSource?: string;
  componentTagName?: string;
  componentBindingName: string;
  componentImportStatement?: string;
};

type ImportedBinding = {
  localName: string;
  source: string;
  statement: string;
};

export function componentNameFromSource(source: string): string {
  const withoutQuery = source.split(/[?#]/, 1)[0] || source;
  const filename = path.basename(withoutQuery);
  const extension = path.extname(filename);
  const stem = extension ? filename.slice(0, -extension.length) : filename;
  const name = toPascalCase(stem);
  return name === "Variant" ? "MuseaComponent" : name;
}

export function resolveArtComponent(
  art: ArtFileInfo,
  filePath: string,
  scriptSetup: ArtScriptSetupInfo,
  options: { root?: string; scanRoots?: string[] } = {},
): ResolvedArtComponent {
  let componentImportPath: string | undefined;
  let componentSource: string | undefined;
  let componentTagName: string | undefined;
  let componentBindingName = "__MuseaComponent";
  const defineArtComponentName = scriptSetup?.defineArtComponentName;
  const defineArtComponentSource = scriptSetup?.defineArtComponentSource;
  const componentImportStatement = scriptSetup?.defineArtComponentImport;

  if (art.isInline && art.componentPath) {
    // Inline art: import the host .vue file itself as the component
    componentImportPath = options.root
      ? (resolveComponentSourcePath(
          art,
          filePath,
          allowedSourceRoots(options.root, options.scanRoots ?? []),
        ) ?? undefined)
      : art.componentPath;
    componentTagName = componentNameFromSource(art.componentPath);
    componentBindingName = componentTagName;
  } else if (defineArtComponentName && defineArtComponentSource) {
    // Standalone art files explicitly import their target and pass that binding
    // to defineArt(). The import is the single source of both runtime identity
    // and component props typing.
    componentSource = defineArtComponentSource;
    if (componentSource) {
      const sourceArt = { ...art, metadata: { ...art.metadata, component: componentSource } };
      componentImportPath = options.root
        ? (resolveComponentSourcePath(
            sourceArt,
            filePath,
            allowedSourceRoots(options.root, options.scanRoots ?? []),
          ) ?? undefined)
        : path.isAbsolute(componentSource)
          ? componentSource
          : path.resolve(path.dirname(filePath), componentSource);
    }
    componentTagName = defineArtComponentName;
    componentBindingName = componentTagName;
  } else {
    const inferred = inferArtComponent(art, scriptSetup?.imports ?? []);
    if (inferred) {
      componentSource = inferred.source;
      const sourceArt = { ...art, metadata: { ...art.metadata, component: inferred.source } };
      componentImportPath = options.root
        ? (resolveComponentSourcePath(
            sourceArt,
            filePath,
            allowedSourceRoots(options.root, options.scanRoots ?? []),
          ) ?? undefined)
        : path.isAbsolute(inferred.source)
          ? inferred.source
          : path.resolve(path.dirname(filePath), inferred.source);
      componentTagName = inferred.localName;
      componentBindingName = inferred.localName;
      return {
        componentImportPath,
        componentSource,
        componentTagName,
        componentBindingName,
        componentImportStatement: inferred.statement,
      };
    }
  }

  return {
    componentImportPath,
    componentSource,
    componentTagName,
    componentBindingName,
    componentImportStatement,
  };
}

/** Infer the demonstrated component from an imported tag shared by every variant. */
export function inferArtComponent(
  art: Pick<ArtFileInfo, "variants">,
  imports: string[],
): ImportedBinding | undefined {
  const imported = imports.flatMap(importedBindings);
  if (imported.length === 0 || art.variants.length === 0) return undefined;

  const tagsByVariant = art.variants.map((variant) => templateTags(variant.template));
  const candidates = imported.filter(({ localName }) =>
    tagsByVariant.every((tags) => tags.has(localName) || tags.has(toKebabComponentName(localName))),
  );
  if (candidates.length === 1) return candidates[0];
  if (candidates.length > 1) throw ambiguousTargetError(candidates);

  // A single imported component used by at least one variant is still
  // unambiguous for partial/empty variants. Multiple matches require the
  // explicit `defineArt(Component)` escape hatch.
  const used = imported.filter(({ localName }) =>
    tagsByVariant.some((tags) => tags.has(localName) || tags.has(toKebabComponentName(localName))),
  );
  if (used.length > 1) throw ambiguousTargetError(used);
  return used[0];
}

function ambiguousTargetError(candidates: ImportedBinding[]): Error {
  const names = candidates.map(({ localName }) => localName).join(", ");
  return new Error(
    `Musea cannot infer one target component (${names}). Add defineArt(Target) to select it explicitly.`,
  );
}

function importedBindings(statement: string): ImportedBinding[] {
  if (/^\s*import\s+type\b/u.test(statement)) return [];
  const normalized = statement.replace(/\s+/gu, " ").trim();
  const match = normalized.match(/^import\s+(.+?)\s+from\s+(["'])([^"']+)\2\s*;?$/u);
  if (!match) return [];
  const clause = match[1].trim();
  const source = match[3];
  const names: string[] = [];
  const defaultName = clause.match(/^([A-Za-z_$][\w$]*)(?:\s*,|$)/u)?.[1];
  if (defaultName) names.push(defaultName);
  const namespaceName = clause.match(/\*\s+as\s+([A-Za-z_$][\w$]*)/u)?.[1];
  if (namespaceName) names.push(namespaceName);
  const named = clause.match(/\{([\s\S]*?)\}/u)?.[1];
  if (named) {
    for (const part of named.split(",")) {
      const value = part.trim().replace(/^type\s+/u, "");
      if (!value || part.trim().startsWith("type ")) continue;
      const local = value.match(/(?:^|\s+as\s+)([A-Za-z_$][\w$]*)$/u)?.[1];
      if (local) names.push(local);
    }
  }
  return [...new Set(names)].map((localName) => ({ localName, source, statement }));
}

function templateTags(template: string): Set<string> {
  const tags = new Set<string>();
  for (const match of template.matchAll(/<([A-Za-z_$][\w$.-]*)(?=[\s/>])/gu)) {
    if (match[1]) tags.add(match[1]);
  }
  return tags;
}

function toKebabComponentName(name: string): string {
  return name.replace(/([a-z0-9])([A-Z])/gu, "$1-$2").toLowerCase();
}
