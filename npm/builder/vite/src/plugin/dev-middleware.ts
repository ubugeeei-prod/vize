import type { ViteDevServer } from "vite";
import fs from "node:fs";
import { normalizeViteDevMiddlewareUrl } from "@vizejs/native";

import type { VizePluginState } from "./state.ts";

export function installVirtualAssetMiddleware(
  devServer: ViteDevServer,
  state: Pick<VizePluginState, "logger">,
): void {
  devServer.middlewares.use((req, _res, next) => {
    const rewrite = req.url ? normalizeViteDevMiddlewareUrl(req.url) : null;
    if (rewrite && fs.existsSync(rewrite.fsPath) && fs.statSync(rewrite.fsPath).isFile()) {
      state.logger.log(`middleware: rewriting ${req.url} -> ${rewrite.cleanedUrl}`);
      req.url = rewrite.cleanedUrl;
    }
    next();
  });
}
