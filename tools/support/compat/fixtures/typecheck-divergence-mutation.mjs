import { writeFileSync } from "node:fs";
import { resolve } from "node:path";

export {
  buildSeededMutation,
  seededMutationDiagnostic,
} from "./typecheck-divergence-mutation-source.mjs";
import { summarizeMutationObservations } from "./typecheck-divergence-mutation-delta.mjs";
import { observeMutationState } from "./typecheck-divergence-mutation-runner.mjs";

export async function executeSeededMutationOracle({
  project,
  fixtureRoot,
  file,
  cleanSource,
  brokenSource,
  diagnostic,
  vizeLaunch,
  vueTsc,
  baselineArgs,
  documentedDifferences,
}) {
  const sourcePath = resolve(fixtureRoot, file);
  let result;
  let primaryError;
  try {
    writeFileSync(sourcePath, cleanSource);
    const clean = await observe({ name: "clean" });
    writeFileSync(sourcePath, brokenSource);
    const broken = await observe({ name: "broken" });
    writeFileSync(sourcePath, cleanSource);
    const repaired = await observe({ name: "repaired" });
    result = summarizeMutationObservations({
      clean,
      broken,
      repaired,
      cleanSource,
      brokenSource,
      diagnostic,
    });
  } catch (error) {
    primaryError = error;
  }

  try {
    restoreSource(sourcePath, cleanSource);
  } catch (restoreError) {
    // A restore failure must not swallow the reason the oracle failed: the
    // shard artifact records that message as `unusableReason`.
    if (primaryError != null) {
      throw new AggregateError(
        [primaryError, restoreError],
        `${errorMessage(primaryError)}; ${errorMessage(restoreError)}`,
      );
    }
    throw restoreError;
  }
  if (primaryError != null) throw primaryError;
  return result;

  async function observe({ name }) {
    return observeMutationState({
      name,
      project,
      fixtureRoot,
      file,
      sourcePath,
      vizeLaunch,
      vueTsc,
      baselineArgs,
      documentedDifferences,
    });
  }
}

function restoreSource(sourcePath, cleanSource) {
  try {
    writeFileSync(sourcePath, cleanSource);
  } catch (error) {
    throw new Error(`Seeded mutation oracle could not restore ${sourcePath}`, { cause: error });
  }
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}
