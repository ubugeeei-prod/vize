import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { object, jsonFile } from "./value.ts";
import { requireCaptureReady } from "./recipes.ts";
import type { Request } from "./model.ts";
import { runPublic } from "./pipeline.ts";
import { loadOriginalFixture } from "./fixtures.ts";
import { mainManualContext } from "./context.ts";
/** Authoring this adapter does not execute it. The current private required-data
 * packets intentionally refuse preflight; a complete manual Actions request
 * is necessary before public network/install/capture effects can begin. */
export function preflight(file: string): void {
  const value = object(jsonFile(file));
  requireCaptureReady(value as unknown as Request);
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const main = async () => {
    const [mode, requestFile, pnpm, output] = process.argv.slice(2);
    if (mode === "--preflight" && requestFile && !pnpm && !output)
      return preflight(resolve(requestFile));
    if (mode !== "--execute-public" || !requestFile || !pnpm || !output)
      throw new Error("--preflight request | --execute-public request pnpm-cli output required");
    const request = object(jsonFile(requestFile)) as unknown as Request;
    mainManualContext();
    await runPublic(request, loadOriginalFixture(request.role), resolve(pnpm), resolve(output));
  };
  main().catch((error: unknown) => {
    process.stderr.write(String(error) + "\n");
    process.exitCode = 1;
  });
}
