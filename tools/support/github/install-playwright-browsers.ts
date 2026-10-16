/**
 * Installs Chromium using the lockfile version owned by a selected workspace.
 *
 * Usage: node tools/support/github/install-playwright-browsers.ts
 *
 * PLAYWRIGHT_WORKSPACE defaults to `./playground`; scoped CSS uses `./tests`,
 * where its runtime oracle imports Playwright. The selected workspace must be
 * installed before calling this helper.
 */
import { isEntrypoint, runCommand, runEntrypoint } from "../command-runtime.ts";

export function main(): number {
  return runCommand("vp", [
    "exec",
    "--filter",
    process.env.PLAYWRIGHT_WORKSPACE || "./playground",
    "--",
    "playwright",
    "install",
    "chromium",
    "--with-deps",
  ]);
}

if (isEntrypoint(import.meta.url)) runEntrypoint(main);
