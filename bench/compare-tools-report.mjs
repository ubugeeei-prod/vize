/**
 * Reporting half of the tool comparison benchmark (bench/compare-tools.mjs).
 *
 * Engine classes are ranked separately (#3283). A surface whose incumbent tool
 * and Vize lane run on different underlying engines — `vue-tsc` on the
 * JavaScript TypeScript compiler versus `vize check` on native tsgo/Corsa —
 * must not publish a single ratio: that ratio measures TypeScript's Go rewrite
 * as much as the Vue layer. Such surfaces publish `speedup: null` with
 * `speedupStatus: "cross-engine"` and are ranked within each engine class
 * instead, mirroring bench/check-gate-report.mjs.
 */

import { ENGINE_CLASSES } from "./check-gate-report.mjs";

export const CROSS_ENGINE_CELL = "n/a (cross-engine)";

/**
 * Which surfaces span engine classes, and which class each of their variants
 * belongs to. Declared here rather than at the measurement site so a recorded
 * artifact can be re-rendered (bench/render-results.mjs) with the same
 * classification that produced it. `large-check` is `check` re-run over one
 * large SFC, so it carries the same variant ids.
 */
export const ENGINE_CLASSES_BY_SURFACE = {
  check: {
    "golar-default": "tsgo-native",
    "golar-typecheck": "tsgo-native",
    "verter-tsc": "tsgo-native",
    "vue-tsc": "typescript-js",
    "vize-check-1t": "tsgo-native",
    "vize-check-max": "tsgo-native",
  },
  "large-check": {
    "golar-default": "tsgo-native",
    "golar-typecheck": "tsgo-native",
    "verter-tsc": "tsgo-native",
    "vue-tsc": "typescript-js",
    "vize-check-1t": "tsgo-native",
    "vize-check-max": "tsgo-native",
  },
};

export function formatSpeedup(value) {
  if (!Number.isFinite(value)) {
    return "n/a";
  }
  return `${value.toFixed(1)}x`;
}

export function getVariant(surface, id) {
  if (!id) {
    return null;
  }
  return surface.variants.find((variant) => variant.id === id) ?? null;
}

function engineClassOf(surface, id) {
  return surface.engineClasses?.[id] ?? null;
}

/**
 * Rank the variants of one engine class fastest-first. Only used for surfaces
 * that declare `engineClasses`; every other surface keeps a single ordering.
 */
export function rankWithinEngineClasses(surface) {
  const classes = surface.engineClasses;
  if (classes == null) {
    return null;
  }
  const grouped = new Map();
  for (const variant of surface.variants) {
    const engineClass = classes[variant.id];
    if (engineClass == null) {
      throw new Error(
        `compare-tools: variant ${variant.id} of surface ${surface.id} has no engine class`,
      );
    }
    if (!grouped.has(engineClass)) {
      grouped.set(engineClass, []);
    }
    grouped.get(engineClass).push(variant);
  }
  return [...grouped.entries()].map(([engineClass, variants]) => {
    const ordered = [...variants].sort((a, b) => a.medianMs - b.medianMs);
    const fastest = ordered[0];
    return {
      engineClass,
      label: ENGINE_CLASSES[engineClass] ?? engineClass,
      rows: ordered.map((variant) => ({
        id: variant.id,
        label: variant.label,
        medianMs: variant.medianMs,
        // Ratio against the fastest row of the SAME engine class, so it is the
        // Vue layer alone and is safe to publish.
        relativeToFastest:
          fastest.medianMs > 0 ? Number((variant.medianMs / fastest.medianMs).toFixed(3)) : null,
      })),
    };
  });
}

/**
 * Attach the primary speedup, refusing to compute one across engine classes.
 */
export function createSurface(surface) {
  const baseline = getVariant(surface, surface.baselineId);
  const vizeMax = getVariant(surface, surface.vizeMaxId);
  const baselineClass = engineClassOf(surface, surface.baselineId);
  const vizeMaxClass = engineClassOf(surface, surface.vizeMaxId);
  const crossEngine =
    baselineClass != null && vizeMaxClass != null && baselineClass !== vizeMaxClass;
  const comparable = baseline != null && vizeMax != null && vizeMax.medianMs > 0;

  return {
    ...surface,
    // `null`, not NaN: NaN serialises to `null` in the JSON artifact anyway, so
    // the in-memory value must say the same thing the artifact says.
    primarySpeedup: crossEngine || !comparable ? null : baseline.medianMs / vizeMax.medianMs,
    speedupStatus: crossEngine ? "cross-engine" : comparable ? "ranked" : "unavailable",
    engineClassRanking: rankWithinEngineClasses(surface),
  };
}

function surfaceSpeedupCell(surface) {
  return surface.speedupStatus === "cross-engine"
    ? CROSS_ENGINE_CELL
    : formatSpeedup(surface.primarySpeedup);
}

export function renderSurfaceTable(surface, formatMs) {
  const baseline = getVariant(surface, surface.baselineId);
  const vizeSingle = getVariant(surface, surface.vizeSingleId);
  const vizeMax = getVariant(surface, surface.vizeMaxId);
  return `| ${surface.label} | ${surface.files.toLocaleString()} | ${baseline?.label ?? "n/a"} | ${formatMs(baseline?.medianMs)} | ${vizeSingle ? formatMs(vizeSingle.medianMs) : "n/a"} | ${formatMs(vizeMax?.medianMs)} | ${surfaceSpeedupCell(surface)} |`;
}

export function renderEngineClassSections(surfaces, formatMs) {
  const lines = [];
  for (const surface of surfaces) {
    if (surface.engineClassRanking == null) {
      continue;
    }
    lines.push(`#### ${surface.label} — engine classes ranked separately`);
    lines.push("");
    lines.push("| Engine class | Row | Median | Relative to fastest in class |");
    lines.push("| --- | --- | ---: | ---: |");
    for (const group of surface.engineClassRanking) {
      for (const row of group.rows) {
        lines.push(
          `| ${group.label} | ${row.label} | ${formatMs(row.medianMs)} | ${row.relativeToFastest == null ? "n/a" : `${row.relativeToFastest.toFixed(2)}x`} |`,
        );
      }
    }
    lines.push("");
    lines.push(
      `No cross-class ratio is published for ${surface.label}: the incumbent runs the JavaScript TypeScript compiler while Vize runs native tsgo, so a single number would credit TypeScript's Go rewrite to the Vue layer.`,
    );
    lines.push("");
  }
  return lines;
}
