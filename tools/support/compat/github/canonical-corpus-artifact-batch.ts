// Validate independent canonical observer artifacts concurrently. Each child
// runs the unchanged `validateObserverArtifact` through the observer CLI and
// returns its receipt, so the finalizer applies exactly the same checks.
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { commandOutput, runChainsSync } from "./canonical-corpus-batch.ts";

const observerScript = fileURLToPath(new URL("./canonical-corpus-observer.mjs", import.meta.url));

export type ArtifactWorker = { readonly observer: string; readonly artifactName: string };

export function validateArtifactsConcurrently(
  workers: readonly ArtifactWorker[],
  root: string,
): unknown[] {
  return runChainsSync(
    workers.map((worker) => [
      {
        file: process.execPath,
        cwd: process.cwd(),
        args: [observerScript, "validate", worker.observer, join(root, worker.artifactName)],
      },
    ]),
  ).map(([result]) => {
    if (!result) throw new Error("Incomplete canonical artifact validation");
    return JSON.parse(commandOutput(result, "utf8")) as unknown;
  });
}
