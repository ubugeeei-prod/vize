import { createHash } from "node:crypto";

export function summarizeMutationObservations({
  clean,
  broken,
  repaired,
  cleanSource,
  brokenSource,
  diagnostic,
}) {
  const empty = emptyDelta();
  const brokenDelta = comparisonDelta(clean.comparison, broken.comparison);
  const repairedDelta = comparisonDelta(clean.comparison, repaired.comparison);
  return {
    cleanExpectedDiagnosticPresent: comparisonHasDiagnostic(clean.comparison, diagnostic),
    expectedDiagnosticMatched: brokenDelta.shared.some((record) =>
      matchesSeededProbe(record, diagnostic),
    ),
    repairedExpectedDiagnosticPresent: comparisonHasDiagnostic(repaired.comparison, diagnostic),
    states: [
      mutationState("clean", cleanSource, clean, empty),
      mutationState("broken", brokenSource, broken, brokenDelta.summary),
      mutationState("repaired", cleanSource, repaired, repairedDelta.summary),
    ],
  };
}

function mutationState(name, source, observed, delta) {
  // `observed.sourceSha256` is the digest of the bytes both checkers actually
  // inspected. Binding it to the planned input here turns a mutation-write or
  // path-selection defect into a hard failure instead of evidence for content
  // no checker ever saw.
  const plannedSha256 = sha256(source);
  if (observed.sourceSha256 !== plannedSha256) {
    throw new Error(
      `Seeded ${name} oracle source digest ${observed.sourceSha256} does not match the planned source digest ${plannedSha256}`,
    );
  }
  return {
    name,
    sourceSha256: observed.sourceSha256,
    ...delta,
    observed: summaryEvidence(observed.comparison.summary),
    vize: runEvidence(observed.vize),
    baseline: runEvidence(observed.baseline),
  };
}

function summaryEvidence(summary) {
  return {
    vizeDiagnosticCount: summary.vizeDiagnosticCount,
    baselineDiagnosticCount: summary.baselineDiagnosticCount,
    sharedCount: summary.sharedCount,
    messageMismatchCount: summary.messageMismatchCount,
    documentedDifferenceCount: summary.documentedDifferenceCount,
    falsePositiveCount: summary.falsePositiveCount,
    falseNegativeCount: summary.falseNegativeCount,
  };
}

function runEvidence(run) {
  return {
    command: run.command,
    exitCode: run.exitCode,
    stdoutSha256: run.stdoutSha256,
    stderrSha256: run.stderrSha256,
  };
}

function comparisonDelta(clean, current) {
  const shared = subtractRecords(current.shared, clean.shared, sharedKey);
  const messageMismatches = subtractRecords(
    current.messageMismatches,
    clean.messageMismatches,
    sharedKey,
  );
  const documentedDifferences = subtractRecords(
    current.documentedDifferences,
    clean.documentedDifferences,
    documentedKey,
  );
  const falsePositives = subtractRecords(
    current.falsePositives,
    clean.falsePositives,
    diagnosticKey,
  );
  const falseNegatives = subtractRecords(
    current.falseNegatives,
    clean.falseNegatives,
    diagnosticKey,
  );
  return {
    shared,
    summary: {
      vizeDiagnosticCount:
        shared.length +
        messageMismatches.length +
        documentedDifferences.length +
        falsePositives.length,
      baselineDiagnosticCount:
        shared.length +
        messageMismatches.length +
        documentedDifferences.length +
        falseNegatives.length,
      sharedCount: shared.length,
      messageMismatchCount: messageMismatches.length,
      documentedDifferenceCount: documentedDifferences.length,
      falsePositiveCount: falsePositives.length,
      falseNegativeCount: falseNegatives.length,
    },
  };
}

function subtractRecords(current, clean, keyOf) {
  const remaining = new Map();
  for (const record of clean) {
    const key = keyOf(record);
    remaining.set(key, (remaining.get(key) ?? 0) + 1);
  }
  return current.filter((record) => {
    const key = keyOf(record);
    const count = remaining.get(key) ?? 0;
    if (count === 0) return true;
    remaining.set(key, count - 1);
    return false;
  });
}

function emptyDelta() {
  return {
    vizeDiagnosticCount: 0,
    baselineDiagnosticCount: 0,
    sharedCount: 0,
    messageMismatchCount: 0,
    documentedDifferenceCount: 0,
    falsePositiveCount: 0,
    falseNegativeCount: 0,
  };
}

function matchesSeededProbe(record, diagnostic) {
  // The planned insertion span is best-effort evidence for the report, not the
  // gate. Some Vue/TypeScript transforms report the same injected TS2322 at a
  // shifted generated coordinate. The oracle already requires exactly one new
  // shared broken diagnostic and a clean repair, so the stable probe identity is
  // the mutated file plus TypeScript error code.
  return (
    record.file === diagnostic.file &&
    record.severity === diagnostic.severity &&
    record.code === diagnostic.code
  );
}

function comparisonHasDiagnostic(comparison, diagnostic) {
  return [
    ...comparison.shared.flatMap((record) => [
      { ...record, message: record.vizeMessage },
      { ...record, message: record.baselineMessage },
    ]),
    ...comparison.messageMismatches.flatMap((record) => [
      { ...record, message: record.vizeMessage },
      { ...record, message: record.baselineMessage },
    ]),
    ...comparison.falsePositives,
    ...comparison.falseNegatives,
  ].some(
    (record) =>
      record.file === diagnostic.file &&
      record.severity === diagnostic.severity &&
      record.line === diagnostic.line &&
      record.column === diagnostic.column &&
      record.code === diagnostic.code &&
      record.message === diagnostic.message,
  );
}

function sharedKey(record) {
  return [
    record.file,
    record.severity,
    record.line,
    record.column,
    record.code,
    record.vizeMessage,
    record.baselineMessage,
  ].join("\0");
}

function documentedKey(record) {
  return [
    record.project,
    record.file,
    record.severity,
    record.line,
    record.column,
    record.vize.code,
    record.vize.message,
    record.baseline.code,
    record.baseline.message,
  ].join("\0");
}

function diagnosticKey(record) {
  return [
    record.file,
    record.severity,
    record.line,
    record.column,
    record.code,
    record.message,
  ].join("\0");
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}
