import { writeFileSync } from "node:fs";
import { resolve } from "node:path";

export {
  buildSeededMutation,
  seededMutationDiagnostic,
} from "./typecheck-divergence-mutation-source.mjs";
import { summarizeMutationObservations } from "./typecheck-divergence-mutation-delta.mjs";
import { observeMutationState } from "./typecheck-divergence-mutation-runner.mjs";

export function executeSeededMutationOracle({
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
    const clean = observe({ name: "clean" });
    writeFileSync(sourcePath, brokenSource);
    const broken = observe({ name: "broken" });
    writeFileSync(sourcePath, cleanSource);
    const repaired = observe({ name: "repaired" });
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

  restoreSource(sourcePath, cleanSource);
  if (primaryError != null) throw primaryError;
  return result;

  function observe({ name }) {
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
