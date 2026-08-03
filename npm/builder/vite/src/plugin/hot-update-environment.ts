import type {
  DevEnvironment,
  EnvironmentModuleNode,
  HmrContext,
  HotUpdateOptions,
  ViteDevServer,
} from "vite";

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
  const modules = await handleHotUpdateHook(
    state,
    { ...options, server } as unknown as HmrContext,
    { requireAcceptingClientModule: true },
  );

  if (modules === undefined && options.file.endsWith(".vue") && state.filter(options.file)) {
    return [];
  }
  return modules as EnvironmentModuleNode[] | void;
}
