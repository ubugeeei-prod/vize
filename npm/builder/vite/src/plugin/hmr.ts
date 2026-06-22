import type { HmrContext, ModuleNode, ViteDevServer } from "vite";
import fs from "node:fs";
import path from "node:path";

import type { VizePluginState } from "./state.ts";
import { getCompileOptionsForRequest } from "./state.ts";
import { compileFile } from "../compiler.ts";
import { detectHmrUpdateType, hasHmrChanges, type HmrUpdateType } from "../hmr.ts";
import { hasDelegatedStyles } from "../utils/index.ts";
import { toPluginVisibleVirtualId, toVirtualId } from "../virtual.ts";
import { resolveCssImports } from "../utils/css.ts";

export const VIZE_COMPONENTS_CSS_BASENAME = "vize-components.css";
export const VIZE_COMPONENTS_CSS_FILE = `assets/${VIZE_COMPONENTS_CSS_BASENAME}`;

type GenerateBundleItem =
  | {
      type: "chunk";
      isEntry?: boolean;
      isDynamicEntry?: boolean;
      viteMetadata?: {
        importedCss?: Set<string>;
      };
    }
  | {
      type?: string;
    };

type GenerateBundle = Record<string, GenerateBundleItem>;

function getVueFilesDependingOn(state: VizePluginState, dependencyFile: string): string[] {
  const normalizedDependency = path.resolve(dependencyFile);
  const owners = new Set<string>();

  for (const cache of [state.cache, state.ssrCache]) {
    for (const [vueFile, compiled] of cache) {
      if (
        compiled.dependencies?.some(
          (dependency) => path.resolve(dependency) === normalizedDependency,
        )
      ) {
        owners.add(vueFile);
      }
    }
  }

  return [...owners];
}

function unique(values: string[]): string[] {
  return [...new Set(values)];
}

function getVueModuleFileCandidates(vueFile: string): string[] {
  return unique([
    toVirtualId(vueFile),
    toPluginVisibleVirtualId(vueFile),
    toVirtualId(vueFile, true),
    toPluginVisibleVirtualId(vueFile, true),
    toPluginVisibleVirtualId(vueFile).split("?")[0],
    vueFile,
  ]);
}

function getStyleModuleFileCandidates(styleId: string): string[] {
  return unique([styleId, `${styleId}.css`]);
}

function collectModulesByFile(server: ViteDevServer, fileIds: readonly string[]): Set<ModuleNode> {
  const modules = new Set<ModuleNode>();

  for (const fileId of fileIds) {
    const matched = server.moduleGraph.getModulesByFile(fileId);
    if (!matched) {
      continue;
    }
    for (const mod of matched) {
      modules.add(mod);
    }
  }

  return modules;
}

function invalidateModules(server: ViteDevServer, modules: Iterable<ModuleNode>): void {
  for (const module of modules) {
    server.moduleGraph.invalidateModule(module);
  }
}

export async function handleHotUpdateHook(
  state: VizePluginState,
  ctx: HmrContext,
): Promise<import("vite").ModuleNode[] | void> {
  const { file, server, read } = ctx;

  const dependencyOwners = getVueFilesDependingOn(state, file);
  if (dependencyOwners.length > 0) {
    const affectedModules: Set<import("vite").ModuleNode> = new Set();

    for (const vueFile of dependencyOwners) {
      state.cache.delete(vueFile);
      state.ssrCache.delete(vueFile);
      state.collectedCss.delete(vueFile);
      state.precompileMetadata.delete(vueFile);
      state.pendingHmrUpdateTypes.set(vueFile, "full-reload");

      const modules = collectModulesByFile(server, getVueModuleFileCandidates(vueFile));

      for (const module of modules) {
        server.moduleGraph.invalidateModule(module);
        affectedModules.add(module);
      }

      state.logger.log(
        `Invalidated ${path.relative(state.root, vueFile)} because ${path.relative(
          state.root,
          file,
        )} changed`,
      );
    }

    return [...affectedModules];
  }

  if (file.endsWith(".vue") && state.filter(file)) {
    try {
      const source = await read();

      const prevCompiled = state.cache.get(file);

      compileFile(file, state.cache, getCompileOptionsForRequest(state, false), source);
      state.ssrCache.delete(file);

      const newCompiled = state.cache.get(file)!;
      try {
        const stat = fs.statSync(file);
        state.precompileMetadata.set(file, {
          mtimeMs: stat.mtimeMs,
          size: stat.size,
        });
      } catch {
        state.precompileMetadata.delete(file);
      }

      if (!hasHmrChanges(prevCompiled, newCompiled)) {
        state.pendingHmrUpdateTypes.delete(file);
        state.logger.log(`Re-compiled: ${path.relative(state.root, file)} (no-op)`);
        return [];
      }

      const updateType: HmrUpdateType = detectHmrUpdateType(prevCompiled, newCompiled);

      state.logger.log(`Re-compiled: ${path.relative(state.root, file)} (${updateType})`);

      const modules = collectModulesByFile(server, getVueModuleFileCandidates(file));

      const hasDelegated = hasDelegatedStyles(newCompiled);

      if (hasDelegated && updateType === "style-only") {
        const affectedModules: Set<import("vite").ModuleNode> = new Set();
        for (const block of newCompiled.styles ?? []) {
          const params = new URLSearchParams();
          params.set("vue", "");
          params.set("type", "style");
          params.set("index", String(block.index));
          if (block.scoped) params.set("scoped", `data-v-${newCompiled.scopeId}`);
          params.set("lang", block.lang ?? "css");
          if (block.module !== false) {
            params.set("module", typeof block.module === "string" ? block.module : "");
          }
          const styleId = `${file}?${params.toString()}`;
          const styleMods = collectModulesByFile(server, getStyleModuleFileCandidates(styleId));
          for (const mod of styleMods) {
            affectedModules.add(mod);
          }
        }
        if (affectedModules.size > 0) {
          return [...affectedModules];
        }
        if (modules.size > 0) {
          return [...modules];
        }
        return [];
      }

      if (updateType === "style-only" && newCompiled.css && !hasDelegated) {
        state.pendingHmrUpdateTypes.delete(file);
        server.ws.send({
          type: "custom",
          event: "vize:update",
          data: {
            id: newCompiled.scopeId,
            type: "style-only",
            css: resolveCssImports(
              newCompiled.css,
              file,
              state.cssAliasRules,
              true,
              state.clientViteBase,
            ),
          },
        });
        return [];
      }

      if (modules.size > 0) {
        state.pendingHmrUpdateTypes.set(file, updateType);
        invalidateModules(server, modules);
        return [...modules];
      }

      state.pendingHmrUpdateTypes.delete(file);
    } catch (e) {
      state.logger.error(`Re-compilation failed for ${file}:`, e);
    }
  }
}

export function handleGenerateBundleHook(
  state: VizePluginState,
  emitFile: (file: { type: "asset"; fileName: string; source: string }) => void,
  bundle: GenerateBundle,
): void {
  if (!state.extractCss || state.collectedCss.size === 0) {
    return;
  }

  let allCss = "";
  for (const css of state.collectedCss.values()) {
    allCss += allCss ? `\n\n${css}` : css;
  }
  if (allCss.trim()) {
    const cssFileName = state.componentsCssFileName || VIZE_COMPONENTS_CSS_FILE;
    emitFile({
      type: "asset",
      fileName: cssFileName,
      source: allCss,
    });
    attachComponentsCssToEntryChunks(bundle, cssFileName);
    state.logger.log(`Extracted CSS to ${cssFileName} (${state.collectedCss.size} components)`);
  }
}

export function resolveComponentsCssFileName(assetsDir: string | undefined): string {
  const normalizedAssetsDir = (assetsDir || "assets").replace(/\\/g, "/").replace(/^\/+|\/+$/g, "");

  if (!normalizedAssetsDir || normalizedAssetsDir === ".") {
    return VIZE_COMPONENTS_CSS_BASENAME;
  }

  return `${normalizedAssetsDir}/${VIZE_COMPONENTS_CSS_BASENAME}`;
}

function attachComponentsCssToEntryChunks(bundle: GenerateBundle, cssFileName: string): void {
  for (const item of Object.values(bundle)) {
    if (item.type !== "chunk" || (!item.isEntry && !item.isDynamicEntry)) {
      continue;
    }

    item.viteMetadata ??= {};
    item.viteMetadata.importedCss ??= new Set<string>();
    item.viteMetadata.importedCss.add(cssFileName);
  }
}
