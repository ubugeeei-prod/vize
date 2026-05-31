import type { HmrContext } from "vite";
import fs from "node:fs";
import path from "node:path";

import type { VizePluginState } from "./state.ts";
import { getCompileOptionsForRequest } from "./state.ts";
import { compileFile } from "../compiler.ts";
import { detectHmrUpdateType, hasHmrChanges, type HmrUpdateType } from "../hmr.ts";
import { hasDelegatedStyles } from "../utils/index.ts";
import { toVirtualId } from "../virtual.ts";
import { resolveCssImports } from "../utils/css.ts";

export const VIZE_COMPONENTS_CSS_FILE = "assets/vize-components.css";

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

export async function handleHotUpdateHook(
  state: VizePluginState,
  ctx: HmrContext,
): Promise<import("vite").ModuleNode[] | void> {
  const { file, server, read } = ctx;

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

      const virtualId = toVirtualId(file);
      const modules =
        server.moduleGraph.getModulesByFile(virtualId) ?? server.moduleGraph.getModulesByFile(file);

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
          const styleMods = server.moduleGraph.getModulesByFile(styleId);
          if (styleMods) {
            for (const mod of styleMods) {
              affectedModules.add(mod);
            }
          }
        }
        if (affectedModules.size > 0) {
          return [...affectedModules];
        }
        if (modules) {
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

      if (modules) {
        state.pendingHmrUpdateTypes.set(file, updateType);
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
    emitFile({
      type: "asset",
      fileName: VIZE_COMPONENTS_CSS_FILE,
      source: allCss,
    });
    attachComponentsCssToEntryChunks(bundle);
    state.logger.log(
      `Extracted CSS to ${VIZE_COMPONENTS_CSS_FILE} (${state.collectedCss.size} components)`,
    );
  }
}

function attachComponentsCssToEntryChunks(bundle: GenerateBundle): void {
  for (const item of Object.values(bundle)) {
    if (item.type !== "chunk" || (!item.isEntry && !item.isDynamicEntry)) {
      continue;
    }

    item.viteMetadata ??= {};
    item.viteMetadata.importedCss ??= new Set<string>();
    item.viteMetadata.importedCss.add(VIZE_COMPONENTS_CSS_FILE);
  }
}
