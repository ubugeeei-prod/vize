import type { Plugin } from "vite";

import { fromPluginVisibleVirtualId } from "../virtual.ts";

const islandFile = /\.island\.vue$/;
const islandModule = /\.island\.vue\?vue&vize(?:-ssr)?(?:&(?:t=\d+|used=true))*$/;

/** Void filters by filename and uses the same filename to parse the island imports. */
export function patchVoidIslandTransform(plugins: readonly Plugin[]): void {
  for (const plugin of plugins) {
    if (plugin.name !== "void-vue:islands") continue;
    const hook = plugin.transform;
    if (!hook || typeof hook !== "object" || !hook.filter || Array.isArray(hook.filter)) continue;
    const { id } = hook.filter;
    if (!(id instanceof RegExp) || id.source !== islandFile.source) continue;

    plugin.transform = {
      ...hook,
      filter: { ...hook.filter, id: [id, islandModule] },
      handler(code, id, ...args) {
        return hook.handler.call(this, code, fromPluginVisibleVirtualId(id) ?? id, ...args);
      },
    };
  }
}
