import type {
  DevEnvironment,
  EnvironmentModuleNode,
  HmrContext,
  HotUpdateOptions,
  ViteDevServer,
} from "vite";

import { toPluginVisibleVirtualId } from "../virtual.ts";
import { handleHotUpdateHook } from "./hmr.ts";
import type { VizePluginState } from "./state.ts";

export async function handleHotUpdateEnvironmentHook(
  state: VizePluginState,
  environment: DevEnvironment,
  options: HotUpdateOptions,
): Promise<EnvironmentModuleNode[] | void> {
  if (environment.name !== "client" && environment.name !== "browser") {
    return options.modules;
  }

  const server = {
    moduleGraph: environment.moduleGraph,
    ws: {
      send: environment.hot.send.bind(environment.hot),
    },
  } as unknown as ViteDevServer;
  let recompileFailed = false;
  const modules = await handleHotUpdateHook(
    state,
    { ...options, server } as unknown as HmrContext,
    {
      ensureAcceptingClientModule: (vueFile) =>
        environment.moduleGraph.ensureEntryFromUrl(toPluginVisibleVirtualId(vueFile), false),
      requireAcceptingClientModule: true,
      onRecompileError: () => {
        recompileFailed = true;
      },
    },
  );

  // A failed re-compilation must keep Vite's default handling so the developer
  // gets an update or an error overlay instead of a silently stale page.
  if (recompileFailed) {
    return undefined;
  }

  if (modules === undefined && options.file.endsWith(".vue") && state.filter(options.file)) {
    return [];
  }
  return modules as EnvironmentModuleNode[] | void;
}
