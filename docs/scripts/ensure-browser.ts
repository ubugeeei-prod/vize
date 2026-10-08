import { spawnSync } from "node:child_process";
import { resolvePuppeteerExecutablePath } from "../browser-path.js";

const browserPath = resolvePuppeteerExecutablePath();

if (browserPath) {
  process.env.PUPPETEER_EXECUTABLE_PATH = browserPath;
  process.stdout.write(`Using browser at ${browserPath}\n`);
  process.exit(0);
}

const result = spawnSync("playwright", ["install", "--with-deps", "chromium"], {
  stdio: "inherit",
  env: process.env,
});

process.exit(result.status ?? 1);
