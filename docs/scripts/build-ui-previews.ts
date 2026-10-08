import { createRequire } from "node:module";
import path from "node:path";

import { resolvePuppeteerExecutablePath } from "../browser-path.js";
import { previewBuildConfig, uiRequire } from "../previews/ui/build-config.ts";
import { captureUiPreviews } from "../previews/ui/capture.ts";

const docsRoot = path.resolve(import.meta.dirname, "..");
const require = createRequire(path.join(docsRoot, "package.json"));
const { build, preview } = (await import(uiRequire.resolve("vite"))) as typeof import("vite");
const config = await previewBuildConfig();
await build(config);
const server = await preview({ ...config, preview: { host: "127.0.0.1", port: 0 } });
try {
  const { chromium } = require("playwright") as typeof import("playwright");
  const address = server.httpServer!.address();
  if (address == null || typeof address === "string")
    throw new Error("Preview server did not bind a local port");
  const browser = await chromium.launch({ executablePath: resolvePuppeteerExecutablePath() });
  try {
    await captureUiPreviews(
      browser,
      `http://127.0.0.1:${address.port}/component-previews/app/`,
      path.join(docsRoot, "public/component-previews"),
    );
  } finally {
    await browser.close();
  }
} finally {
  await new Promise<void>((resolve, reject) =>
    server.httpServer!.close((error) => (error ? reject(error) : resolve())),
  );
}
