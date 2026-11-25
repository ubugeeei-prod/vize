import type { ModuleNode, ViteDevServer } from "vite";
import path from "node:path";
import { createStyleVirtualId } from "./style-request.ts";

import {
  fromPluginVisibleVirtualId,
  isPluginVisibleSsrVirtualId,
  isVizeSsrVirtual,
  isVizeVirtual,
  toPluginVisibleVirtualId,
  toVirtualId,
} from "../virtual.ts";

function unique(values: string[]): string[] {
  return [...new Set(values)];
}

function toViteFsFileId(fileId: string): string | null {
  if (!path.isAbsolute(fileId)) return null;
  const normalized = fileId.replace(/\\/g, "/");
  return normalized.startsWith("/") ? `/@fs${normalized}` : `/@fs/${normalized}`;
}

// The spellings depend on the path alone; a hot update asks for every owner.
const vueModuleFileCandidates = new Map<string, readonly string[]>();

export function getVueModuleFileCandidates(vueFile: string): readonly string[] {
  const known = vueModuleFileCandidates.get(vueFile);
  if (known) return known;
  const candidates = [
    toVirtualId(vueFile),
    toPluginVisibleVirtualId(vueFile),
    toVirtualId(vueFile, true),
    toPluginVisibleVirtualId(vueFile, true),
    toPluginVisibleVirtualId(vueFile).split("?")[0],
    vueFile,
    `${vueFile}.ts`,
    `${vueFile}.ts?vue&vize`,
    `${vueFile}.ts?vue&vize-ssr`,
  ];
  const viteFsCandidates = candidates.flatMap((candidate) => {
    const viteFsId = toViteFsFileId(candidate);
    return viteFsId ? [viteFsId] : [];
  });
  const spellings = unique([...candidates, ...viteFsCandidates]);
  vueModuleFileCandidates.set(vueFile, spellings);
  return spellings;
}

export function getStyleModuleFileCandidates(styleId: string): string[] {
  return unique([styleId, `${styleId}.css`, createStyleVirtualId(styleId)]);
}

/**
 * The graph's modules for any of `fileIds`, read from its id, URL and file
 * indexes. Nothing is resolved: for these absolute candidates
 * `getModuleByUrl` resolves only to pick the URL's extension, which they
 * already carry, and resolving them ran every plugin's `resolveId` per
 * candidate per owner, serially, in each environment's hot update.
 */
export function collectModulesByFile(
  server: ViteDevServer,
  fileIds: readonly string[],
): Set<ModuleNode> {
  const graph = server.moduleGraph as ViteDevServer["moduleGraph"] & {
    urlToModuleMap?: { get(url: string): ModuleNode | undefined };
  };
  const modules = new Set<ModuleNode>();
  const add = (module: ModuleNode | undefined) => {
    if (module) modules.add(module);
  };
  for (const fileId of fileIds) {
    add(graph.getModuleById?.(fileId));
    add(graph.urlToModuleMap?.get(fileId));
    for (const module of graph.getModulesByFile(fileId) ?? []) add(module);
  }
  return modules;
}

function isClientVirtualId(id: string): boolean {
  if (isVizeVirtual(id)) return !isVizeSsrVirtual(id);
  return fromPluginVisibleVirtualId(id) !== null && !isPluginVisibleSsrVirtualId(id);
}

/**
 * The compiled client modules among `modules`. `byId` also accepts a module
 * whose id is the compiled client module although it was first requested by
 * the plain `.vue` URL: it is the module `ensureEntryFromUrl` would return.
 */
export function preferAcceptingClientModules(
  modules: Set<ModuleNode>,
  requireAcceptingClientModule = false,
  byId = false,
): Set<ModuleNode> {
  const accepting = [...modules].filter(
    (module) =>
      isClientVirtualId(module.url) || (byId && !!module.id && isClientVirtualId(module.id)),
  );
  if (accepting.length > 0) return new Set(accepting);
  return requireAcceptingClientModule ? new Set() : modules;
}

export function invalidateModules(server: ViteDevServer, modules: Iterable<ModuleNode>): void {
  for (const module of modules) server.moduleGraph.invalidateModule(module);
}
