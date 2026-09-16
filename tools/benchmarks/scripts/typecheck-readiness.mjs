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
    const previous = new Set(baseline.diagnostics.map(JSON.stringify));
    const current = new Set(unchanged.map(JSON.stringify));
    throw new Error(
      `missed the corpus-scale plant or changed baseline diagnostics: ${JSON.stringify({
        plants: additions,
        added: unchanged.filter((d) => !previous.has(JSON.stringify(d))).slice(0, 3),
        removed: baseline.diagnostics.filter((d) => !current.has(JSON.stringify(d))).slice(0, 3),
      })}`,
    );
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

/** Validate every lane before warmup; rejected lanes cannot publish timings. */
export function gateTypecheckVariants(variants, checkDir, prepareProject, onRejected) {
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
          corpusBaselines.set(variant.id, error);
        }
      }
    });
    for (const dir of [...Object.values(plants.dirs), corpus.dir]) prepareProject(dir);
    return variants.flatMap((variant) => {
      try {
        if (corpusBaselines.get(variant.id) instanceof Error) {
          throw corpusBaselines.get(variant.id);
        }
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
        if (onRejected) {
          onRejected(variant, error, "preflight");
          return [];
        }
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

export const OPTIONAL_TYPECHECK_VARIANTS = new Set(["golar-typecheck", "golar-default"]);

export function recordTypecheckRejection(rejected, variant, error, phase) {
  if (!OPTIONAL_TYPECHECK_VARIANTS.has(variant.id)) throw error;
  rejected.push({ id: variant.id, label: variant.label, phase, reason: error.message });
}
