/**
 * Reporting half of the tool comparison benchmark (tools/benchmarks/scripts/compare-tools.mjs).
 *
 * Every surface publishes its ratio against the incumbent it declares — the
 * tool a reader is actually running today. For type checking that is
 * `vue-tsc`: it is what Vue projects use, so a speedup measured against
 * anything else answers a question nobody asked. An earlier revision retargeted
 * the type-check ratio at `verter-tsc` because that tool drives the same native
 * tsgo binary Vize does, which isolates the Vue layer; but verter-tsc is an
 * experimental checker almost nobody runs, so the headline number described a
 * comparison its readers could not make.
 *
 * The attribution problem that motivated the retarget is real and is disclosed
 * instead of hidden: `vue-tsc` runs the JavaScript TypeScript compiler while
 * `vize check` runs native tsgo/Corsa, so part of that ratio is TypeScript's Go
 * rewrite rather than the Vue layer. `engineClasses` keeps every row ranked
 * inside its own engine class (#3283), so the same-engine rows below the table
 * still show the Vue layer alone, and `engineClassNote` says which number is
 * which. A surface whose declared incumbent did not resolve publishes no ratio
 * (`speedupStatus: "unavailable"`).
 */

import { ENGINE_CLASSES } from "./check-gate-report.mjs";
import { OPTIONAL_TYPECHECK_VARIANTS } from "./typecheck-readiness.mjs";

/**
 * Which surfaces span engine classes, and which class each of their variants
 * belongs to. Declared here rather than at the measurement site so a recorded
 * artifact can be re-rendered (tools/benchmarks/scripts/render-results.mjs) with the same
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

function assertRequiredEngineVariants(surface) {
  const expected = ENGINE_CLASSES_BY_SURFACE[surface.id];
  if (expected == null && !surface.requireEngineVariants) {
    return;
  }
  if (expected == null) {
    throw new Error(
      `compare-tools: ${surface.id} requested engine-variant coverage but has no engine class map`,
    );
  }
  const actualIds = new Set(surface.variants.map((variant) => variant.id));
  for (const rejected of surface.rejectedVariants ?? []) {
    if (
      !OPTIONAL_TYPECHECK_VARIANTS.has(rejected.id) ||
      actualIds.has(rejected.id) ||
      !["preflight", "warmup", "measure"].includes(rejected.phase) ||
      typeof rejected.label !== "string" ||
      !rejected.label.trim() ||
      typeof rejected.reason !== "string" ||
      !rejected.reason.trim()
    )
      throw new Error(`compare-tools: invalid rejected type-check variant: ${rejected.id}`);
    actualIds.add(rejected.id);
  }
  const missing = Object.keys(expected).filter((id) => !actualIds.has(id));
  if (missing.length > 0) {
    throw new Error(
      `compare-tools: ${surface.id} is missing required engine-class variants: ${missing.join(", ")}`,
    );
  }
}

export function formatSpeedup(value) {
  if (!Number.isFinite(value)) {
    return "n/a";
  }
  if (value > 0 && value < 1) {
    const rounded = value.toFixed(2);
    return `${Number(rounded) === 0 || Number(rounded) === 1 ? value : rounded}x`;
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
 * Pick the row the published ratio is measured against: always the declared
 * incumbent. `crossEngine` reports whether that row runs a different engine
 * from the Vize lane, which decides what the note under the table has to say,
 * not whether a ratio appears.
 */
function resolveSpeedupBaseline(surface) {
  const vizeMaxClass = engineClassOf(surface, surface.vizeMaxId);
  const declaredClass = engineClassOf(surface, surface.baselineId);
  return {
    crossEngine: declaredClass != null && vizeMaxClass != null && declaredClass !== vizeMaxClass,
    baseline: getVariant(surface, surface.baselineId),
  };
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
  assertRequiredEngineVariants(surface);
  const { requireEngineVariants: _requireEngineVariants, ...outputSurface } = surface;
  const vizeMax = getVariant(surface, surface.vizeMaxId);
  const comparable = vizeMax != null && vizeMax.medianMs > 0;
  const { crossEngine, baseline } = resolveSpeedupBaseline(surface);
  const ranked = comparable && baseline != null && baseline.medianMs > 0;
  let speedupStatus = "unavailable";
  if (ranked) {
    speedupStatus = crossEngine ? "cross-engine" : "ranked";
  }

  return {
    ...outputSurface,
    // `null`, not NaN: NaN serialises to `null` in the JSON artifact anyway, so
    // the in-memory value must say the same thing the artifact says.
    primarySpeedup: ranked ? baseline.medianMs / vizeMax.medianMs : null,
    // The row the ratio is against, so a reader never has to infer whether it
    // came from the declared incumbent or from the in-class one.
    speedupBaselineId: ranked ? baseline.id : null,
    speedupStatus,
    engineClassRanking: rankWithinEngineClasses(surface),
  };
}

function surfaceSpeedupCell(surface) {
  return formatSpeedup(surface.primarySpeedup);
}

export function renderSurfaceTable(surface, formatMs) {
  // The ratio and the two medians beside it must be the same comparison, so
  // the row follows the published baseline rather than the declared one.
  const baseline = getVariant(surface, surface.speedupBaselineId ?? surface.baselineId);
  const vizeSingle = getVariant(surface, surface.vizeSingleId);
  const vizeMax = getVariant(surface, surface.vizeMaxId);
  return `| ${surface.label} | ${surface.files.toLocaleString()} | ${baseline?.label ?? "n/a"} | ${formatMs(baseline?.medianMs)} | ${vizeSingle ? formatMs(vizeSingle.medianMs) : "n/a"} | ${formatMs(vizeMax?.medianMs)} | ${surfaceSpeedupCell(surface)} |`;
}

/**
 * The sentence under a cross-engine surface's ranking: what the published
 * ratio does and does not attribute to the Vue layer.
 */
function engineClassNote(surface) {
  const published = getVariant(surface, surface.speedupBaselineId);
  if (published == null) {
    return `No ratio is published for ${surface.label}: its declared incumbent did not produce a timing in this run.`;
  }
  return `The ${surface.label} ratio compares Vize with ${published.label}, the checker Vue projects run today. ${published.label} drives the JavaScript TypeScript compiler while Vize drives native tsgo, so that ratio is the whole toolchain and not the Vue layer alone; the per-engine-class rows above isolate the Vue layer by ranking each class against its own fastest row. Diagnostic coverage can differ between tools; no ratio here is an accuracy-parity claim.`;
}

export const ENGINE_CLASS_TEXT = {
  heading: (surface) => `${surface.label} — engine classes ranked separately`,
  columns: ["Engine class", "Row", "Median", "Relative to fastest in class"],
  group: (group) => group.label,
  note: engineClassNote,
  rejected: (variant) =>
    `${variant.label}: rejected during ${variant.phase}; no timing or rank published.`,
  validation:
    "Type-check work validation (strict templates; before warmup; diagnostics rechecked on every run):",
  proofColumns: [
    "Row",
    "Minimal plants",
    "Corpus plant",
    "Diagnostics",
    "Exit status",
    "Diagnostic SHA-256",
  ],
  passed: "passed",
  failed: "failed",
};

export function renderEngineClassSections(surfaces, formatMs, t = ENGINE_CLASS_TEXT) {
  const lines = [];
  for (const surface of surfaces) {
    if (surface.engineClassRanking == null) {
      continue;
    }
    lines.push(`#### ${t.heading(surface)}`);
    lines.push("");
    lines.push(`| ${t.columns.join(" | ")} |`);
    lines.push("| --- | --- | ---: | ---: |");
    for (const group of surface.engineClassRanking) {
      for (const row of group.rows) {
        lines.push(
          `| ${t.group(group)} | ${row.label} | ${formatMs(row.medianMs)} | ${row.relativeToFastest == null ? "n/a" : `${row.relativeToFastest.toFixed(2)}x`} |`,
        );
      }
    }
    lines.push("");
    lines.push(t.note(surface));
    lines.push("");
    for (const rejected of surface.rejectedVariants ?? []) {
      lines.push(t.rejected(rejected));
      lines.push("");
      lines.push("```text", rejected.reason.replaceAll("```", "'''"), "```", "");
    }
    const checked = surface.variants.filter((variant) => variant.correctness != null);
    if (checked.length > 0) {
      lines.push(t.validation);
      lines.push("");
      lines.push(`| ${t.proofColumns.join(" | ")} |`);
      lines.push("| --- | ---: | --- | ---: | ---: | --- |");
      for (const variant of checked) {
        const proof = variant.correctness;
        lines.push(
          `| ${variant.label} | ${proof.minimalPlants.length} | ${proof.corpusPlant ? t.passed : t.failed} | ${proof.diagnosticCount} | ${proof.status} | \`${proof.diagnosticFingerprint}\` |`,
        );
      }
      lines.push("");
    }
  }
  return lines;
}
