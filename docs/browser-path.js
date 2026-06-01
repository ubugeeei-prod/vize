import { existsSync } from "node:fs";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

const browserCandidates = [
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  "/Applications/Chromium.app/Contents/MacOS/Chromium",
  "/usr/bin/google-chrome-stable",
  "/usr/bin/google-chrome",
  "/usr/bin/chromium-browser",
  "/usr/bin/chromium",
];

export function resolvePuppeteerExecutablePath(env = process.env) {
  const configuredPath = env.PUPPETEER_EXECUTABLE_PATH;
  if (configuredPath && existsSync(configuredPath)) {
    return configuredPath;
  }

  try {
    const { chromium } = require("playwright");
    const executablePath = chromium?.executablePath?.();
    if (executablePath && existsSync(executablePath)) {
      return executablePath;
    }
  } catch {
    // Fall through to common system browser paths.
  }

  return browserCandidates.find((candidate) => existsSync(candidate));
}
