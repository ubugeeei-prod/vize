import fs from "node:fs";
import path from "node:path";
import { classifyVitePluginRequest } from "@vizejs/native";

import { parseCustomBlockQuery } from "../custom-blocks.ts";
import type { CustomBlockInfo } from "../sfc-types.ts";
import { getCompiledStyleSource } from "./load-style.ts";
import type { VizePluginState } from "./state.ts";

export interface CustomBlockLoadOptions {
  ssr?: boolean;
  addWatchFile?: (file: string) => void;
}

/**
 * Raw source for `?vue&type=<block>&index=N`, or `null` when `id` is not a
 * custom-block query. Style, script, and template queries stay on their own
 * paths. A `src` block reads that file and watches it.
 */
export function loadCustomBlock(
  state: VizePluginState,
  id: string,
  options?: CustomBlockLoadOptions,
): { code: string; map: null } | null {
  const query = parseCustomBlockQuery(id);
  if (!query) return null;
  const request = classifyVitePluginRequest(id);
  if (!request.isVueSfcPath) return null;

  const owner = vueOwnerPath(state, request.path);
  const compiled = getCompiledStyleSource(state, owner, !!options?.ssr);
  const block = compiled?.customBlocks?.[query.index];
  if (!block) return { code: "", map: null };
  if (block.src) return loadSrcBlock(state, owner, block, query.type, options?.addWatchFile);
  return { code: block.content, map: null };
}

function vueOwnerPath(state: VizePluginState, requestPath: string): string {
  const owner = requestPath.startsWith("/@fs/") ? requestPath.slice("/@fs".length) : requestPath;
  if (owner.startsWith("/") && !fs.existsSync(owner)) {
    const projectPath = path.resolve(state.root, owner.slice(1));
    if (fs.existsSync(projectPath)) return projectPath;
  }
  return owner;
}

function loadSrcBlock(
  state: VizePluginState,
  owner: string,
  block: CustomBlockInfo,
  blockType: string,
  addWatchFile: ((file: string) => void) | undefined,
): { code: string; map: null } {
  const blockPath = path.resolve(path.dirname(owner), block.src ?? "");
  addWatchFile?.(blockPath);
  state.server?.watcher.add(blockPath);
  try {
    return { code: fs.readFileSync(blockPath, "utf8"), map: null };
  } catch {
    throw new Error(
      `[vize] Custom block <${blockType} src="${block.src}"> not found (resolved: ${blockPath}) in ${owner}`,
    );
  }
}
