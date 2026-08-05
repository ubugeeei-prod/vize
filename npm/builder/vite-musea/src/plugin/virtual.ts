/**
 * Virtual module handling for the Musea Vite plugin.
 *
 * Contains `resolveId`, `load`, and `handleHotUpdate` hooks that
 * manage virtual modules for gallery, manifest, preview, and art files.
 */

import path from "node:path";
import type { ModuleNode } from "vite";

import type { ArtFileInfo } from "../types/index.js";
import type { MuseaVueVersion } from "../types/plugin.js";

import { generateGalleryModule } from "../gallery/module.js";
import { generatePreviewModule } from "../preview/index.js";
import { generateManifestModule } from "../manifest.js";
import { generateArtModule } from "../art-module.js";
import { compileVariantSfc } from "../art-variant-sfc.js";
import { generateSharedSetupModule, sharedBindingNames } from "../art-shared-setup.js";
import { parseScriptSetupForArt } from "../art-module.js";
import { toPascalCase } from "../utils.js";

// Virtual module prefixes
const VIRTUAL_MUSEA_PREFIX = "\0musea:";
const VIRTUAL_GALLERY = "\0musea-gallery";
const VIRTUAL_MANIFEST = "\0musea-manifest";

export { VIRTUAL_MUSEA_PREFIX, VIRTUAL_GALLERY, VIRTUAL_MANIFEST };

/**
 * Shared state required by virtual module hooks.
 */
export interface VirtualModuleState {
  basePath: string;
  inlineArt: boolean;
  artFiles: Map<string, ArtFileInfo>;
  resolvedPreviewCss: string[];
  resolvedPreviewSetup: string | null;
  getConfigRoot: () => string;
  getScanRoots: () => string[];
  getVueVersion: () => MuseaVueVersion | undefined;
  getServer: () => {
    moduleGraph: { getModulesByFile(id: string): Set<ModuleNode> | undefined };
  } | null;
  processArtFile: (filePath: string) => Promise<void>;
}

export function createResolveId(state: VirtualModuleState) {
  return function resolveId(id: string): string | null {
    const root = state.getConfigRoot();

    if (id === VIRTUAL_GALLERY) {
      return VIRTUAL_GALLERY;
    }
    if (id === VIRTUAL_MANIFEST) {
      return VIRTUAL_MANIFEST;
    }
    // Handle virtual:musea-preview: prefix for preview modules
    if (id.startsWith("virtual:musea-preview:")) {
      return "\0musea-preview:" + id.slice("virtual:musea-preview:".length);
    }
    // One module per variant, compiled through the SFC pipeline (#3857). Each
    // one is a complete module with its own default export, so they cannot be
    // concatenated into the art module.
    if (id.startsWith("virtual:musea-variant:")) {
      return "\0musea-variant:" + id.slice("virtual:musea-variant:".length) + "?musea-virtual";
    }
    // One shared setup instance for art files that opt out of per-variant
    // isolation, so `scriptSetupIsolated: false` keeps meaning one instance.
    if (id.startsWith("virtual:musea-shared:")) {
      return "\0musea-shared:" + id.slice("virtual:musea-shared:".length) + "?musea-virtual";
    }
    // Handle virtual:musea-art: prefix for preview modules
    // Append ?musea-virtual to prevent other plugins (e.g. unplugin-vue-i18n)
    // from treating .vue-ending virtual IDs as Vue SFC files
    if (id.startsWith("virtual:musea-art:")) {
      const artPath = id.slice("virtual:musea-art:".length);
      if (state.artFiles.has(artPath)) {
        return "\0musea-art:" + artPath + "?musea-virtual";
      }
    }
    if (id.endsWith(".art.vue")) {
      const resolved = path.resolve(root, id);
      if (state.artFiles.has(resolved)) {
        return VIRTUAL_MUSEA_PREFIX + resolved + "?musea-virtual";
      }
    }
    // Inline art: resolve .vue files that have <art> blocks
    if (state.inlineArt && id.endsWith(".vue") && !id.endsWith(".art.vue")) {
      const resolved = path.resolve(root, id);
      if (state.artFiles.has(resolved)) {
        return VIRTUAL_MUSEA_PREFIX + resolved + "?musea-virtual";
      }
    }
    return null;
  };
}

export function createLoad(state: VirtualModuleState) {
  return function load(id: string): string | null {
    if (id === VIRTUAL_GALLERY) {
      return generateGalleryModule(state.basePath);
    }
    if (id === VIRTUAL_MANIFEST) {
      return generateManifestModule(state.artFiles);
    }
    // Handle \0musea-preview: prefix for preview modules
    if (id.startsWith("\0musea-preview:")) {
      const rest = id.slice("\0musea-preview:".length);
      const lastColonIndex = rest.lastIndexOf(":");
      if (lastColonIndex !== -1) {
        const artPath = rest.slice(0, lastColonIndex);
        const variantName = rest.slice(lastColonIndex + 1);
        const art = state.artFiles.get(artPath);
        if (art) {
          const variantComponentName = toPascalCase(variantName);
          return generatePreviewModule(
            art,
            variantComponentName,
            variantName,
            state.resolvedPreviewCss,
            state.resolvedPreviewSetup,
            state.getVueVersion(),
          );
        }
      }
    }
    if (id.startsWith("\0musea-shared:")) {
      const artPath = id.slice("\0musea-shared:".length).replace(/\?musea-virtual$/, "");
      const art = state.artFiles.get(artPath);
      if (art) return generateSharedSetupModule(art, artPath);
    }
    if (id.startsWith("\0musea-variant:")) {
      const rest = id.slice("\0musea-variant:".length).replace(/\?musea-virtual$/, "");
      const lastColonIndex = rest.lastIndexOf(":");
      if (lastColonIndex !== -1) {
        const artPath = rest.slice(0, lastColonIndex);
        const variantName = rest.slice(lastColonIndex + 1);
        const art = state.artFiles.get(artPath);
        const variant = art?.variants.find((candidate) => candidate.name === variantName);
        if (art && variant) {
          const parsedScriptSetup = art.scriptSetupContent
            ? parseScriptSetupForArt(art.scriptSetupContent)
            : null;
          const shared =
            art.scriptSetupIsolated === false && parsedScriptSetup
              ? {
                  moduleId: `virtual:musea-shared:${artPath}`,
                  names: sharedBindingNames(parsedScriptSetup),
                }
              : undefined;
          const compiled = compileVariantSfc(art, variant.template, variant.name, artPath, {
            scriptSetup: art.scriptSetupContent,
            parsedScriptSetup,
            root: state.getConfigRoot(),
            scanRoots: state.getScanRoots(),
            sharedBindings: shared,
          });
          if (compiled.errors.length > 0) {
            throw new Error(
              `Failed to compile <art> variant "${variantName}" in ${artPath}:\n${compiled.errors.join("\n")}`,
            );
          }
          return compiled.code;
        }
      }
    }
    // Handle \0musea-art: prefix for preview modules
    if (id.startsWith("\0musea-art:")) {
      const artPath = id.slice("\0musea-art:".length).replace(/\?musea-virtual$/, "");
      const art = state.artFiles.get(artPath);
      if (art) {
        return generateArtModule(art, artPath, {
          root: state.getConfigRoot(),
          scanRoots: state.getScanRoots(),
          vueVersion: state.getVueVersion(),
        });
      }
    }
    if (id.startsWith(VIRTUAL_MUSEA_PREFIX)) {
      const realPath = id.slice(VIRTUAL_MUSEA_PREFIX.length).replace(/\?musea-virtual$/, "");
      const art = state.artFiles.get(realPath);
      if (art) {
        return generateArtModule(art, realPath, {
          root: state.getConfigRoot(),
          scanRoots: state.getScanRoots(),
          vueVersion: state.getVueVersion(),
        });
      }
    }
    return null;
  };
}

export function createHandleHotUpdate(state: VirtualModuleState) {
  return async function handleHotUpdate(ctx: { file: string }): Promise<ModuleNode[] | undefined> {
    const { file } = ctx;
    if (file.endsWith(".art.vue") && state.artFiles.has(file)) {
      await state.processArtFile(file);

      // Invalidate virtual modules
      const virtualId = VIRTUAL_MUSEA_PREFIX + file + "?musea-virtual";
      const server = state.getServer();
      const modules = server?.moduleGraph.getModulesByFile(virtualId);
      if (modules) {
        return [...modules];
      }
    }

    // Inline art: HMR for .vue files with <art> blocks
    if (
      state.inlineArt &&
      file.endsWith(".vue") &&
      !file.endsWith(".art.vue") &&
      state.artFiles.has(file)
    ) {
      await state.processArtFile(file);

      const virtualId = VIRTUAL_MUSEA_PREFIX + file;
      const server = state.getServer();
      const modules = server?.moduleGraph.getModulesByFile(virtualId);
      if (modules) {
        return [...modules];
      }
    }

    return undefined;
  };
}
