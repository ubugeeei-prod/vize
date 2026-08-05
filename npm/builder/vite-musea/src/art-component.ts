/**
 * Resolve which component an art file demonstrates, and under what tag.
 *
 * Shared by the art module and by the per-variant SFC compiler (#3857) so both
 * agree on the import path, the tag `<Self>` expands to, and the binding name.
 * The parsed `<script setup>` is passed in rather than parsed here, so this
 * module does not have to import back from `art-module.ts`.
 */

import path from "node:path";

import { allowedSourceRoots, resolveComponentSourcePath } from "./component-source.js";
import type { ArtFileInfo } from "./types/index.js";
import { toPascalCase } from "./utils.js";

export type ArtScriptSetupInfo = {
  defineArtComponentName?: string;
  defineArtComponentSource?: string;
} | null;

export type ResolvedArtComponent = {
  componentImportPath?: string;
  componentTagName?: string;
  componentBindingName: string;
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
  let componentTagName: string | undefined;
  let componentBindingName = "__MuseaComponent";
  const defineArtComponentName = scriptSetup?.defineArtComponentName;
  const defineArtComponentSource = scriptSetup?.defineArtComponentSource;

  if (art.isInline && art.componentPath) {
    // Inline art: import the host .vue file itself as the component
    componentImportPath = options.root
      ? (resolveComponentSourcePath(
          art,
          filePath,
          allowedSourceRoots(options.root, options.scanRoots ?? []),
        ) ?? undefined)
      : art.componentPath;
    componentTagName = "MuseaComponent";
  } else if (defineArtComponentSource || art.metadata.component) {
    // .art.vue: resolve component from defineArt(source, ...) or the legacy component attribute.
    const componentSource = defineArtComponentSource ?? art.metadata.component;
    if (componentSource) {
      const sourceArt =
        componentSource === art.metadata.component
          ? art
          : { ...art, metadata: { ...art.metadata, component: componentSource } };
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
    componentTagName =
      defineArtComponentName ??
      (art.metadata.component ? componentNameFromSource(art.metadata.component) : "MuseaComponent");
    componentBindingName = componentTagName;
  }

  return { componentImportPath, componentTagName, componentBindingName };
}

/** Expand `<Self>` in a variant template to the resolved component tag. */
export function expandSelfTag(template: string, componentTagName: string | undefined): string {
  if (!componentTagName) return template;
  return template
    .replace(/<Self/g, `<${componentTagName}`)
    .replace(/<\/Self>/g, `</${componentTagName}>`);
}
