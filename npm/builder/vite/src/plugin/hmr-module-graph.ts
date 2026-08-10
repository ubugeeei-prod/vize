import type { ModuleNode, ViteDevServer } from "vite";
import path from "node:path";

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

export function getVueModuleFileCandidates(vueFile: string): string[] {
  const candidates = [
    toVirtualId(vueFile),
    toPluginVisibleVirtualId(vueFile),
    toVirtualId(vueFile, true),
    toPluginVisibleVirtualId(vueFile, true),
    toPluginVisibleVirtualId(vueFile).split("?")[0],
    vueFile,
  ];
  const viteFsCandidates = candidates.flatMap((candidate) => {
    const viteFsId = toViteFsFileId(candidate);
    return viteFsId ? [viteFsId] : [];
  });
  return unique([...candidates, ...viteFsCandidates]);
}

export function getStyleModuleFileCandidates(styleId: string): string[] {
  return unique([styleId, `${styleId}.css`]);
}

export async function collectModulesByFile(
  server: ViteDevServer,
  fileIds: readonly string[],
): Promise<Set<ModuleNode>> {
  const modules = new Set<ModuleNode>();
  for (const fileId of fileIds) {
    const add = (module: ModuleNode | undefined) => {
      if (module) modules.add(module);
    };
    add(server.moduleGraph.getModuleById?.(fileId));
    if (!fileId.startsWith("\0")) {
      try {
        add(await server.moduleGraph.getModuleByUrl?.(fileId));
      } catch {
        // A speculative ID that no plugin claims must not abort HMR.
      }
    }
    for (const module of server.moduleGraph.getModulesByFile(fileId) ?? []) add(module);
  }
  return modules;
}

export function preferAcceptingClientModules(
  modules: Set<ModuleNode>,
  requireAcceptingClientModule = false,
): Set<ModuleNode> {
  const accepting = [...modules].filter((module) => {
    if (isVizeVirtual(module.url)) return !isVizeSsrVirtual(module.url);
    return (
      fromPluginVisibleVirtualId(module.url) !== null && !isPluginVisibleSsrVirtualId(module.url)
    );
  });
  if (accepting.length > 0) return new Set(accepting);
  return requireAcceptingClientModule ? new Set() : modules;
}

export function invalidateModules(server: ViteDevServer, modules: Iterable<ModuleNode>): void {
  for (const module of modules) server.moduleGraph.invalidateModule(module);
}
