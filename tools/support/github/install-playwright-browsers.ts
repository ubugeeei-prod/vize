/**
 * Installs the Chromium browser used by the playground browser test jobs.
 *
 * Usage: node tools/support/github/install-playwright-browsers.ts
 *
 * The installer runs through the `./playground` workspace package, which owns
 * the browser tests. After `vp install`, that package has a local `playwright`
 * binary that `vp exec --filter` resolves deterministically, and the browser
 * version stays aligned with the lockfile without parsing it here.
 */
import { isEntrypoint, runCommand, runEntrypoint } from "../command-runtime.ts";

export function main(): number {
  return runCommand("vp", [
    "exec",
    "--filter",
    "./playground",
    "--",
    "playwright",
    "install",
    "chromium",
    "--with-deps",
  ]);
}

if (isEntrypoint(import.meta.url)) runEntrypoint(main);
