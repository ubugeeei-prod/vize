/**
 * Musea surface for the tool comparison benchmark (#3464).
 *
 * The other surfaces in `tools/benchmarks/scripts/compare-tools.mjs` rank Vize against an
 * incumbent tool. This one has no incumbent: nothing else in a Vite build does
 * what `@vizejs/vite-plugin-musea` does, and Storybook — the closest thing to a
 * comparable — is a whole gallery framework with its own builder rather than a
 * plugin whose hooks can be measured side by side. So the surface publishes
 * `speedup: null` and reports absolute per-stage cost, which is the number a
 * regression actually moves. `createSurface` already renders that honestly:
 * with no baseline variant it reports `speedupStatus: "unavailable"` rather
 * than inventing a ratio.
 *
 * Kept out of `tools/benchmarks/scripts/compare-tools.mjs` so that file, which is long past the
 * repository's per-file line budget, does not grow another lane's worth.
 */

import { createSurface } from "./compare-tools-report.mjs";
import { MUSEA_CORPUS_FILE_COUNT } from "./musea-corpus.mjs";
import { measureMusea } from "./musea.mjs";

export const DEFAULT_MUSEA_FILE_COUNT = MUSEA_CORPUS_FILE_COUNT;

/**
 * Throughput for one stage.
 *
 * Carries an M tier that the shared formatter in `tools/benchmarks/scripts/compare-tools.mjs`
 * does not need: the musea-nuxt stage answers thousands of cheap specifier
 * lookups per pass and lands in the millions per second, which the k tier
 * alone renders as an unreadable "8640.0k".
 */
function throughputOf(units, unitLabel, medianMs) {
  if (!Number.isFinite(medianMs) || medianMs <= 0) {
    return "n/a";
  }
  const perSecond = (units / medianMs) * 1000;
  if (perSecond >= 1_000_000) {
    return `${(perSecond / 1_000_000).toFixed(1)}M ${unitLabel}/s`;
  }
  if (perSecond >= 1000) {
    return `${(perSecond / 1000).toFixed(1)}k ${unitLabel}/s`;
  }
  return `${perSecond.toFixed(0)} ${unitLabel}/s`;
}

/**
 * Shape measured stages into the surface, without measuring anything.
 *
 * Separated from `measureMuseaSurface` so the published shape can be asserted
 * in a clean checkout, where none of the packages the lane measures are built.
 * What matters about that shape is what it refuses to say: with no baseline
 * variant `createSurface` reports `primarySpeedup: null` and
 * `speedupStatus: "unavailable"` instead of inventing a ratio, and the surface
 * stores no historical value, so nothing downstream can read a drift number
 * out of it. Enforcement stays with benchmark.yml's fixed-baseline schedule
 * (#3586) rather than a second gate growing here.
 */
export function buildMuseaSurface(data) {
  const artifacts = Object.fromEntries(
    Object.entries(data.artifacts ?? {}).map(([label, artifact]) => [
      label,
      { sha256: artifact.sha256, pinned: artifact.pinned },
    ]),
  );
  return createSurface({
    id: "musea",
    label: "Musea plugin hooks (art gallery build)",
    files: data.fileCount,
    bytes: data.bytes,
    variants: data.stages.map((stage) => ({
      id: stage.id,
      label: stage.label,
      medianMs: stage.medianMs,
      msPerUnit: stage.msPerUnit,
      runs: stage.runs,
      units: stage.units,
      unitLabel: stage.unitLabel,
      digest: stage.digest,
      throughput: throughputOf(stage.units, stage.unitLabel, stage.medianMs),
    })),
    // No incumbent tool performs this work, so there is nothing to rank against.
    baselineId: null,
    vizeSingleId: null,
    vizeMaxId: "musea-plugin-total",
    variantCount: data.variantCount,
    artifacts,
  });
}

/**
 * Measure the Musea plugin surface.
 *
 * `options.runs` and `options.warmups` are the same knobs every other surface
 * uses, so one `--runs` flag governs the whole comparison run.
 */
export async function measureMuseaSurface(rootDir, options) {
  return buildMuseaSurface(
    await measureMusea({
      root: rootDir,
      files: options.museaFileCount ?? DEFAULT_MUSEA_FILE_COUNT,
      runs: options.runs,
      warmups: options.warmups,
    }),
  );
}
