import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { ensureTargetBuilds } from "./dev-app/builds.ts";
import { runtimeOptions } from "./dev-app/env.ts";
import { createLaunchConfig, startForeground } from "./dev-app/launch.ts";

export { buildMisskeyDevConfig, runMisskeyBeforeStart } from "./dev-app/misskey.ts";
export { resolveAvailablePort } from "./dev-app/ports.ts";

const entryFilename = fileURLToPath(import.meta.url);

/**
 * Starts a selected real-world dev application from the MoonBit `dev_app`
 * wrapper.
 *
 * The implementation is intentionally a thin coordinator. Build policy, target
 * launch configuration, Misskey preflight work, and port probing live in smaller
 * modules under `support/dev-app/` so each behavior can carry focused
 * documentation and tests without turning this entry point into a kitchen sink.
 */
export async function main(): Promise<never> {
  const launchConfig = await createLaunchConfig(runtimeOptions.target);

  ensureTargetBuilds(runtimeOptions.target, runtimeOptions.skipBuild);

  if (!runtimeOptions.skipSetup) {
    launchConfig.setup?.();
  }

  launchConfig.beforeStart?.();
  return await startForeground(launchConfig);
}

function isMainModule(): boolean {
  const entryPath = process.argv[1];
  if (entryPath == null) {
    return false;
  }
  return path.resolve(entryPath) === entryFilename;
}

if (isMainModule()) {
  await main();
}
