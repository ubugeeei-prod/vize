import type { ModuleNode, ViteDevServer } from "vite";

import { customBlockQuery } from "../custom-blocks.ts";
import type { CompiledModule } from "../types.ts";
import { collectModulesByFile } from "./hmr-module-graph.ts";

/** Module ids for one custom block, including Vite's `vue=` rewrite and `/@fs`. */
export function customBlockModuleIds(file: string, compiled: CompiledModule): string[] {
  const ids: string[] = [];
  for (const [index, block] of (compiled.customBlocks ?? []).entries()) {
    const query = customBlockQuery(block, index);
    const bare = `${file}?${query}`;
    const rewritten = `${file}?${query.replace(/^vue&/, "vue=&")}`;
    ids.push(bare, rewritten);
    if (bare.startsWith("/")) {
      ids.push(`/@fs${bare}`, `/@fs${rewritten}`);
    }
  }
  return ids;
}

/** Custom-block modules whose raw content a dev edit must send through again. */
export async function collectCustomBlockModules(
  server: ViteDevServer,
  file: string,
  compiled: CompiledModule,
): Promise<Set<ModuleNode>> {
  return collectModulesByFile(server, customBlockModuleIds(file, compiled));
}
