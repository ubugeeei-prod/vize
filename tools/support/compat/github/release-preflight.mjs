// Compatibility for historical imports; Actions executes the TypeScript entrypoint.
import { fileURLToPath } from "node:url";
import path from "node:path";
import { runReleasePreflightCli } from "./release-preflight.ts";
export * from "./release-preflight.ts";
if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1]))
  runReleasePreflightCli();
