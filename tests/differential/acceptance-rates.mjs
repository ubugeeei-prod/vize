import assert from "node:assert/strict";
import { classifyDialectCoverage, classifyNativeRow, validateResultEnvelope } from "./harness.mjs";

function emptyCounts() {
  return {
    planned: 0,
    nativeHandled: 0,
    nativeEquivalent: 0,
    unsupported: 0,
    legacyBacked: 0,
    unverified: 0,
  };
}

function addCounts(counts, state, equivalent) {
  counts.planned += 1;
  if (state === "native-handled") {
    counts.nativeHandled += 1;
    if (equivalent) counts.nativeEquivalent += 1;
  } else if (state === "unsupported") {
    counts.unsupported += 1;
  } else if (state === "legacy-backed") {
    counts.legacyBacked += 1;
  } else {
    counts.unverified += 1;
  }
}

function dialectBucket(buckets, dialect) {
  if (!buckets[dialect]) buckets[dialect] = emptyCounts();
  return buckets[dialect];
}

// Product adapters validate their runtime observations, receipts and comparisons.
// This report only credits the exact, pinned case/target coordinates they expose.
export function summarizeNativeAcceptance(loaded, report, options) {
  assert.match(options.sourceRevision, /^[a-f0-9]{40}$/, "expected source revision is required");
  validateResultEnvelope(loaded, report, options.sourceRevision);
  const targets = new Map();
  const plannedTargets = new Map(loaded.cases.map((fixture) => [fixture.id, fixture.targets]));
  const plannedCases = new Map(loaded.cases.map((fixture) => [fixture.id, fixture]));
  const total = emptyCounts();
  const dialectCoverage = {
    verified: {},
    provisional: {},
    unknown: emptyCounts(),
  };
  for (const row of report.rows) {
    const choices = plannedTargets.get(row.id);
    const target = row.target ?? choices[0];
    if (!targets.has(target)) targets.set(target, emptyCounts());
    const counts = targets.get(target);
    const state = classifyNativeRow(row, {
      sourceRevision: options.sourceRevision,
      buildReceiptSha256:
        typeof options.buildReceiptSha256 === "function"
          ? options.buildReceiptSha256(row, target)
          : options.buildReceiptSha256,
      requiredStages:
        typeof options.requiredStages === "function"
          ? options.requiredStages(row, target)
          : options.requiredStages,
      verifyObservation: (observation) =>
        options.verifyObservation?.(observation, row, target) === true,
    });
    const equivalent =
      state === "native-handled" &&
      row.comparison.state === "equal" &&
      options.verifyComparison?.(row, target) === true;
    for (const bucket of [counts, total]) addCounts(bucket, state, equivalent);
    const fixtureCoverage = classifyDialectCoverage(plannedCases.get(row.id).dialectCoverage);
    if (fixtureCoverage.readiness === "unknown") {
      // A provisional label remains in the unknown readiness denominator.
      addCounts(dialectCoverage.unknown, state, equivalent);
    }
    if (fixtureCoverage.measurement !== "unknown") {
      for (const dialect of fixtureCoverage.dialects) {
        addCounts(
          dialectBucket(dialectCoverage[fixtureCoverage.measurement], dialect),
          state,
          equivalent,
        );
      }
    }
  }
  return {
    schema: "vize.differential.acceptance",
    version: 1,
    scope: "registered-fixtures",
    product: report.product,
    sourceRevision: report.sourceRevision,
    manifestSha256: report.manifestSha256,
    plannedCases: loaded.cases.length,
    total,
    targets: Object.fromEntries([...targets].sort(([a], [b]) => a.localeCompare(b))),
    dialectCoverage: {
      verified: Object.fromEntries(
        Object.entries(dialectCoverage.verified).sort(([a], [b]) => a.localeCompare(b)),
      ),
      provisional: Object.fromEntries(
        Object.entries(dialectCoverage.provisional).sort(([a], [b]) => a.localeCompare(b)),
      ),
      unknown: dialectCoverage.unknown,
    },
  };
}

export function renderNativeAcceptanceMarkdown(summaries) {
  const lines = [
    "### Native-only acceptance (registered fixtures)",
    "",
    "Product | Target | Native-only | Equivalent | Unsupported | Legacy-backed | Unverified",
    "--- | --- | ---: | ---: | ---: | ---: | ---:",
  ];
  for (const summary of summaries) {
    for (const [target, counts] of Object.entries(summary.targets)) {
      const fraction = `${counts.nativeHandled}/${counts.planned}`;
      lines.push(
        `${summary.product} | ${target} | ${fraction} (${((100 * counts.nativeHandled) / counts.planned).toFixed(1)}%) | ${counts.nativeEquivalent}/${counts.planned} | ${counts.unsupported} | ${counts.legacyBacked} | ${counts.unverified}`,
      );
    }
  }
  lines.push(
    "",
    "Product | Dialect evidence | Dialect | Planned rows | Native-only | Equivalent | Unsupported | Legacy-backed | Unverified",
    "--- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---:",
  );
  for (const summary of summaries) {
    for (const evidence of ["verified", "provisional"]) {
      for (const [dialect, counts] of Object.entries(summary.dialectCoverage[evidence])) {
        lines.push(
          `${summary.product} | ${evidence} | ${dialect} | ${counts.planned} | ${counts.nativeHandled} | ${counts.nativeEquivalent} | ${counts.unsupported} | ${counts.legacyBacked} | ${counts.unverified}`,
        );
      }
    }
    const unknown = summary.dialectCoverage.unknown;
    lines.push(
      `${summary.product} | readiness | unknown | ${unknown.planned} | ${unknown.nativeHandled} | ${unknown.nativeEquivalent} | ${unknown.unsupported} | ${unknown.legacyBacked} | ${unknown.unverified}`,
    );
  }
  lines.push(
    "",
    "Denominator: every planned case/target coordinate in each registered manifest. Provisional labels remain in the unknown readiness row until verified execution evidence exists. Equivalent requires a product comparator. Unregistered products and unlisted dialects are unmeasured.",
    "",
  );
  return lines.join("\n");
}
