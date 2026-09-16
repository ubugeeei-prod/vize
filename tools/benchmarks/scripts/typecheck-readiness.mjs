import { readFileSync } from "node:fs";
import { join } from "node:path";

import { resolveVuePackageDir } from "./check-gate-env.mjs";
import {
  CORPUS_PLANT_FILE,
  MINIMAL_PLANTS,
  prepareCorpusPlant,
  prepareMinimalPlants,
} from "./check-gate-plants.mjs";
import { diagnosticFingerprint, normalizeTypecheckResult } from "./typecheck-command.mjs";

export function assertMinimalPlant(report, plant) {
  const line = plant.expected.diagnostic.split(":")[1];
  const codes = plant.id === "templateEvent" ? ["TS2322", "TS2345"] : ["TS2322"];
  if (
    ![1, 2].includes(report.status) ||
    report.diagnostics.length !== 1 ||
    !report.diagnostics.some(
      ([file, kind, actualLine, , code]) =>
        file === plant.expected.file &&
        kind === "error" &&
        actualLine === line &&
        codes.includes(code),
    )
  ) {
    throw new Error(`missed the ${plant.label} plant`);
  }
}

export function assertCorpusPlant(baseline, planted) {
  const additions = planted.diagnostics.filter((d) => d[0] === CORPUS_PLANT_FILE);
  const unchanged = planted.diagnostics.filter((d) => d[0] !== CORPUS_PLANT_FILE);
  if (
    ![1, 2].includes(planted.status) ||
    additions.length !== 2 ||
    !["2", "7"].every((line) =>
      additions.some(
        ([, kind, actualLine, , code]) =>
          kind === "error" && actualLine === line && code === "TS2322",
      ),
    ) ||
    JSON.stringify(unchanged) !== JSON.stringify(baseline.diagnostics)
  ) {
    throw new Error("missed the corpus-scale plant or changed baseline diagnostics");
  }
}

/** Each timed invocation must do the same checked work as the un-timed baseline. */
export function checkedMeasurement(variant, cwd, baseline) {
  const expected = diagnosticFingerprint(baseline);
  return () => {
    const result = variant.run(cwd);
    const report = normalizeTypecheckResult(result, cwd, variant.format);
    if (diagnosticFingerprint(report) !== expected) {
      throw new Error(`${variant.id}: diagnostics changed during measurement`);
    }
    if (!Number.isFinite(result.ms) || result.ms <= 0) {
      throw new Error(`${variant.id}: invalid measured duration`);
    }
    return result.ms;
  };
}

/** Validate every lane before warmup; a failure prevents publishing the artifact. */
export function gateTypecheckVariants(variants, checkDir, prepareProject) {
  const plants = prepareMinimalPlants(`${checkDir}-readiness`, resolveVuePackageDir());
  let corpus;
  try {
    const tsconfig = JSON.parse(readFileSync(join(checkDir, "tsconfig.json"), "utf8"));
    const corpusBaselines = new Map();
    // Some checkers name virtual files with a hash of their absolute path.
    // Compare before/after planting at the SAME copy path, without erasing hashes.
    corpus = prepareCorpusPlant(checkDir, tsconfig, (dir) => {
      prepareProject(dir);
      for (const variant of variants) {
        try {
          corpusBaselines.set(
            variant.id,
            normalizeTypecheckResult(variant.run(dir), dir, variant.format),
          );
        } catch (error) {
          throw new Error(
            `${variant.id}: refusing to publish a type-check timing: ${error.message}`,
            { cause: error },
          );
        }
      }
    });
    for (const dir of [...Object.values(plants.dirs), corpus.dir]) prepareProject(dir);
    return variants.map((variant) => {
      try {
        const run = (dir) => normalizeTypecheckResult(variant.run(dir), dir, variant.format);
        for (const plant of MINIMAL_PLANTS) assertMinimalPlant(run(plants.dirs[plant.id]), plant);
        const baseline = run(checkDir);
        assertCorpusPlant(corpusBaselines.get(variant.id), run(corpus.dir));
        return {
          ...variant,
          measure: checkedMeasurement(variant, checkDir, baseline),
          correctness: {
            minimalPlants: MINIMAL_PLANTS.map((plant) => plant.id),
            corpusPlant: true,
            strictTemplates: true,
            status: baseline.status,
            diagnosticCount: baseline.diagnostics.length,
            diagnosticFingerprint: diagnosticFingerprint(baseline),
            corpusBaselineFingerprint: diagnosticFingerprint(corpusBaselines.get(variant.id)),
          },
        };
      } catch (error) {
        throw new Error(
          `${variant.id}: refusing to publish a type-check timing: ${error.message}`,
          {
            cause: error,
          },
        );
      }
    });
  } finally {
    plants.cleanup();
    corpus?.cleanup();
  }
}
