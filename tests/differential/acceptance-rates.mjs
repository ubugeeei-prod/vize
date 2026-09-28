import assert from "node:assert/strict";
import { classifyNativeRow, validateResultEnvelope } from "./harness.mjs";

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

// Product adapters validate their runtime observations, receipts and comparisons.
// This report only credits the exact, pinned case/target coordinates they expose.
export function summarizeNativeAcceptance(loaded, report, options) {
  assert.match(options.sourceRevision, /^[a-f0-9]{40}$/, "expected source revision is required");
  validateResultEnvelope(loaded, report, options.sourceRevision);
  const targets = new Map();
  const plannedTargets = new Map(loaded.cases.map((fixture) => [fixture.id, fixture.targets]));
  const total = emptyCounts();
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
    for (const bucket of [counts, total]) {
      bucket.planned += 1;
      if (state === "native-handled") {
        bucket.nativeHandled += 1;
        if (equivalent) bucket.nativeEquivalent += 1;
      } else if (state === "unsupported") {
        bucket.unsupported += 1;
      } else if (state === "legacy-backed") {
        bucket.legacyBacked += 1;
      } else {
        // A failed, skipped, missing or unverified execution stays in the denominator.
        bucket.unverified += 1;
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
    "Denominator: every planned case/target coordinate in each registered manifest. Equivalent requires a product comparator. Unregistered products and unlisted dialects are unmeasured.",
    "",
  );
  return lines.join("\n");
}
