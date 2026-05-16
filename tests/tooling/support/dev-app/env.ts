import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { normalizeTarget } from "./types.ts";

const moduleFilename = fileURLToPath(import.meta.url);
const moduleDirname = path.dirname(moduleFilename);

/**
 * Absolute repository root for dev-app tooling.
 *
 * This module lives in `tests/tooling/support/dev-app/`, so four `..` segments
 * land back at the repository root. Computing the path from `import.meta.url`
 * keeps the launcher independent from whichever directory Vite+ used to start
 * the Node process.
 */
export const REPO_ROOT = path.resolve(moduleDirname, "../../../..");

const vitePlusBin = `${process.env.HOME ?? ""}/.vite-plus/bin`;

/**
 * Baseline environment for every subprocess started by the dev-app helper.
 *
 * The Vite+ installer used in CI places `vp` under the user-local Vite+ bin
 * directory, while local development often resolves it from the regular PATH.
 * Prefixing the path here makes both cases behave the same without baking CI
 * paths into individual command invocations.
 */
export const BASE_ENV = {
  ...process.env,
  PATH: `${vitePlusBin}:${process.env.PATH ?? ""}`,
};

/**
 * Runtime options forwarded by the MoonBit wrapper.
 *
 * Reading these values in one place makes tests able to exercise pure helpers
 * without also triggering subprocess startup side effects.
 */
export const runtimeOptions = {
  target: normalizeTarget(process.env.usage_target),
  skipSetup: process.env.usage_skip_setup === "true",
  skipBuild: process.env.usage_skip_build === "true",
};
