import * as vite from "vite";
import type { ResolvedConfig } from "vite";

import type { VizePluginState } from "./state.ts";

interface ViteFileAccessApi {
  isFileLoadingAllowed?: (config: ResolvedConfig, filePath: string) => boolean;
  isFileServingAllowed?: (config: ResolvedConfig, url: string) => boolean;
}

/**
 * Whether the dev server may hand `file` to a browser, by Vite's own
 * `server.fs` rules: the allow list, the deny list, and the files a served
 * module imports. Vite applies them in its fallback `load`, which a plugin that
 * loads the file itself never reaches.
 *
 * A build reads what its graph names, and so does the server's own SSR graph,
 * which Vite does not check either.
 */
export function createServedFileCheck(viteApi: ViteFileAccessApi) {
  return (state: Pick<VizePluginState, "server">, file: string, ssr: boolean): boolean => {
    // `ssr` is the environment's, never the id's: a browser chooses the id.
    const config: Partial<ResolvedConfig> | undefined = ssr ? undefined : state.server?.config;
    const fsOptions = config?.server?.fs;
    if (!config || !fsOptions) return true;
    // Vite compares normalized absolute paths.
    const normalized = file.replaceAll("\\", "/");
    if (typeof viteApi.isFileLoadingAllowed === "function") {
      return viteApi.isFileLoadingAllowed(config as ResolvedConfig, normalized);
    }
    if (typeof viteApi.isFileServingAllowed === "function") {
      return viteApi.isFileServingAllowed(config as ResolvedConfig, normalized);
    }
    return (
      !fsOptions.strict ||
      fsOptions.allow.some(
        (dir) => normalized === dir || normalized.startsWith(dir.endsWith("/") ? dir : `${dir}/`),
      )
    );
  };
}

export const isServedFileAllowed = createServedFileCheck(vite);

/**
 * The directories a served stylesheet may read files from: Vite's strict
 * `server.fs.allow` list. `undefined` for a build, the server's SSR graph, or
 * a server without strict file rules, which read what the stylesheet names.
 */
export function servedStylesheetRoots(
  state: Pick<VizePluginState, "server">,
  ssr: boolean,
): readonly string[] | undefined {
  const fsOptions = ssr ? undefined : state.server?.config?.server?.fs;
  return fsOptions?.strict ? fsOptions.allow : undefined;
}
