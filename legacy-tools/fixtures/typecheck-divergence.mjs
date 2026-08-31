import { createHash } from "node:crypto";
import { isAbsolute } from "node:path";

import {
  collectVizeDiagnostics,
  collectVueTscDiagnostics,
  invalid,
  normalizePath,
} from "./typecheck-divergence-input.mjs";

/**
 * `documentedDifferences` is the reviewed ledger of expected vize-versus-vue-tsc
 * differences (tests/_fixtures/compat-documented-differences.json). An entry can
 * only ever cancel exactly one false positive against exactly one false negative
 * that share a file, severity, line and column, or one message mismatch at that
 * span: both tools must already report something there, and both messages must
 * match the ledger byte for byte after whitespace normalization. A vize-only or
 * vue-tsc-only diagnostic can therefore never be absorbed, and a divergence that
 * shifts position falls straight back into the false-positive/false-negative
 * buckets.
 *
 * Two diagnostics that agree on (file, severity, line, column, code) are only
 * `shared` when their messages are also identical after whitespace
 * normalization; otherwise they land in `messageMismatches`, so a divergence
 * that lives purely in the text is visible to the ratchet instead of being
 * scored as a match (#3447). The comparison is deliberately byte-for-byte and
 * carries no display-normalization rules: every one of the 47 shared pairs the
 * per-PR probes produce already matches exactly, so there is no observed
 * checker-rendering artifact to excuse, and inventing a rule for an unobserved
 * one would loosen the gate on speculation. A rendering difference that does
 * show up surfaces in the bucket with both texts, and is then either recorded
 * in the ledger with a written reason or fixed.
 */
export function compareTypecheckDiagnostics({
  projectId,
  cwd,
  vizeReport,
  vueTscOutput,
  documentedDifferences = [],
}) {
  if (typeof projectId !== "string" || projectId.length === 0) invalid("project id is required");
  if (typeof cwd !== "string" || !isAbsolute(cwd)) invalid("cwd must be absolute");
  const expectedDifferences = selectDocumentedDifferences(documentedDifferences, projectId, cwd);
  const vizeInput = collectVizeDiagnostics(vizeReport, cwd);
  const comparableVueFiles = comparableVueFileSet(vizeReport, cwd);
  const baselineInput = collectVueTscDiagnostics(vueTscOutput, cwd, { comparableVueFiles });
  const vize = vizeInput.diagnostics;
  const baseline = baselineInput.diagnostics;
  const vizeGroups = groupByIdentity(vize);
  const baselineGroups = groupByIdentity(baseline);
  const identities = [...new Set([...vizeGroups.keys(), ...baselineGroups.keys()])].sort(byteOrder);
  const shared = [];
  const messageMismatches = [];
  const falsePositives = [];
  const falseNegatives = [];

  for (const identity of identities) {
    const candidates = vizeGroups.get(identity) ?? [];
    const expected = baselineGroups.get(identity) ?? [];
    const commonCount = Math.min(candidates.length, expected.length);
    for (let index = 0; index < commonCount; index += 1) {
      const candidate = candidates[index];
      const pair = {
        file: candidate.file,
        severity: candidate.severity,
        line: candidate.line,
        column: candidate.column,
        code: candidate.code,
        vizeMessage: candidate.message,
        baselineMessage: expected[index].message,
      };
      if (pair.vizeMessage === pair.baselineMessage) shared.push(pair);
      else messageMismatches.push(pair);
    }
    falsePositives.push(...candidates.slice(commonCount));
    falseNegatives.push(...expected.slice(commonCount));
  }

  shared.sort(compareShared);
  messageMismatches.sort(compareShared);
  falsePositives.sort(compareRecords);
  falseNegatives.sort(compareRecords);
  const documented = pairDocumentedDifferences(
    expectedDifferences,
    falsePositives,
    falseNegatives,
    messageMismatches,
  );
  const classified = {
    shared,
    messageMismatches,
    falsePositives,
    falseNegatives,
    documentedDifferences: documented,
  };
  const summary = {
    vizeDiagnosticCount: vize.length,
    baselineDiagnosticCount: baseline.length,
    sharedCount: shared.length,
    messageMismatchCount: messageMismatches.length,
    documentedDifferenceCount: documented.length,
    falsePositiveCount: falsePositives.length,
    falseNegativeCount: falseNegatives.length,
    falsePositiveRatio: ratio(falsePositives.length, vize.length),
    falseNegativeRatio: ratio(falseNegatives.length, baseline.length),
    vizeExcludedNonVueCount: vizeInput.excludedNonVueCount,
    baselineExcludedNonVueCount: baselineInput.excludedNonVueCount,
    baselineExcludedProjectCount: baselineInput.excludedProjectCount,
    baselineExcludedExternalCount: baselineInput.excludedExternalCount,
    baselineExcludedSupportVueCount: baselineInput.excludedSupportVueCount,
  };
  return {
    schema: "vize.fixtureTypecheckDivergence",
    version: 4,
    project: projectId,
    summary,
    ...classified,
    sha256: createHash("sha256")
      .update(JSON.stringify({ summary, ...classified }))
      .digest("hex"),
  };
}

function comparableVueFileSet(vizeReport, cwd) {
  const files = Array.isArray(vizeReport.files) ? vizeReport.files : [];
  const count = Number.isSafeInteger(vizeReport.fileCount) ? vizeReport.fileCount : files.length;
  return new Set(
    files
      .slice(0, count)
      .map((entry, index) => normalizePath(entry?.file, cwd, `Vize files[${index}].file`))
      .filter((file) => file.endsWith(".vue")),
  );
}
function selectDocumentedDifferences(values, projectId, cwd) {
  if (!Array.isArray(values)) invalid("documented differences must be an array");
  const selected = [];
  const identities = new Set();
  for (const [index, value] of values.entries()) {
    const label = `documented difference ${index}`;
    if (value == null || typeof value !== "object") invalid(`${label} must be an object`);
    if (typeof value.project !== "string" || value.project.length === 0) {
      invalid(`${label} must name a project`);
    }
    const file = normalizePath(value.file, cwd, `${label}.file`);
    if (!file.endsWith(".vue")) invalid(`${label} must reference a .vue file`);
    if (value.severity !== "error" && value.severity !== "warning") {
      invalid(`${label}.severity must be error or warning`);
    }
    const severity = value.severity;
    const line = positiveInteger(value.line, `${label}.line`);
    const column = positiveInteger(value.column, `${label}.column`);
    const vize = documentedSide(value.vize, `${label}.vize`);
    const baseline = documentedSide(value.baseline, `${label}.baseline`);
    if (vize.code === baseline.code && vize.message === baseline.message) {
      invalid(`${label} must record a difference between the two tools`);
    }
    if (!Number.isSafeInteger(value.issue) || value.issue < 1) {
      invalid(`${label}.issue must be the tracking issue number`);
    }
    // A ledger entry suppresses a real divergence, so it has to carry a written
    // rationale a reviewer can check rather than a placeholder.
    const reason = typeof value.reason === "string" ? value.reason.replace(/\s+/g, " ").trim() : "";
    if (reason.length < 40) invalid(`${label}.reason must explain why the difference is expected`);
    const identity = [value.project, file, severity, line, column].join("\0");
    if (identities.has(identity)) invalid(`${label} duplicates an earlier documented difference`);
    identities.add(identity);
    if (value.project !== projectId) continue;
    selected.push({ file, severity, line, column, vize, baseline, issue: value.issue, reason });
  }
  return selected.sort(compareDocumented);
}

function pairDocumentedDifferences(expected, falsePositives, falseNegatives, messageMismatches) {
  const paired = [];
  for (const difference of expected) {
    // A reviewed difference either splits across the two one-sided buckets
    // (the tools disagree on the code, so neither diagnostic has a partner) or
    // sits in the message-mismatch bucket (they agree on the code and differ
    // only in wording). Exactly one of the two shapes may cancel it.
    const positiveIndex = findDocumented(falsePositives, difference, difference.vize);
    const negativeIndex = findDocumented(falseNegatives, difference, difference.baseline);
    if (positiveIndex >= 0 && negativeIndex >= 0) {
      falsePositives.splice(positiveIndex, 1);
      falseNegatives.splice(negativeIndex, 1);
      paired.push(difference);
      continue;
    }
    const mismatchIndex = findDocumentedMismatch(messageMismatches, difference);
    if (mismatchIndex < 0) continue;
    messageMismatches.splice(mismatchIndex, 1);
    paired.push(difference);
  }
  return paired;
}

function findDocumented(records, difference, side) {
  return records.findIndex(
    (candidate) =>
      candidate.file === difference.file &&
      candidate.severity === difference.severity &&
      candidate.line === difference.line &&
      candidate.column === difference.column &&
      candidate.code === side.code &&
      candidate.message === side.message,
  );
}

function findDocumentedMismatch(records, difference) {
  return records.findIndex(
    (candidate) =>
      candidate.file === difference.file &&
      candidate.severity === difference.severity &&
      candidate.line === difference.line &&
      candidate.column === difference.column &&
      candidate.code === difference.vize.code &&
      candidate.code === difference.baseline.code &&
      candidate.vizeMessage === difference.vize.message &&
      candidate.baselineMessage === difference.baseline.message,
  );
}

function documentedSide(value, label) {
  if (value == null || typeof value !== "object") invalid(`${label} must be an object`);
  const code = positiveInteger(value.code, `${label}.code`);
  if (typeof value.message !== "string") invalid(`${label}.message must be a string`);
  const message = value.message.replace(/\s+/g, " ").trim();
  if (message.length === 0) invalid(`${label}.message must be non-empty`);
  return { code, message };
}

function positiveInteger(value, label) {
  if (!Number.isSafeInteger(value) || value < 1)
    invalid(`${label} must be a positive safe integer`);
  return value;
}
function groupByIdentity(records) {
  const groups = new Map();
  for (const value of records) {
    const identity = [value.file, value.severity, value.line, value.column, value.code].join("\0");
    const group = groups.get(identity) ?? [];
    group.push(value);
    groups.set(identity, group);
  }
  for (const group of groups.values()) group.sort(compareRecords);
  return groups;
}

function compareRecords(left, right) {
  return (
    byteOrder(left.file, right.file) ||
    byteOrder(left.severity, right.severity) ||
    left.line - right.line ||
    left.column - right.column ||
    left.code - right.code ||
    byteOrder(left.message, right.message)
  );
}

function compareDocumented(left, right) {
  return (
    byteOrder(left.file, right.file) ||
    byteOrder(left.severity, right.severity) ||
    left.line - right.line ||
    left.column - right.column ||
    left.vize.code - right.vize.code ||
    left.baseline.code - right.baseline.code
  );
}

function compareShared(left, right) {
  return (
    byteOrder(left.file, right.file) ||
    byteOrder(left.severity, right.severity) ||
    left.line - right.line ||
    left.column - right.column ||
    left.code - right.code ||
    byteOrder(left.vizeMessage, right.vizeMessage) ||
    byteOrder(left.baselineMessage, right.baselineMessage)
  );
}

function byteOrder(left, right) {
  if (left === right) return 0;
  const leftCodePoints = left[Symbol.iterator]();
  const rightCodePoints = right[Symbol.iterator]();
  while (true) {
    const leftPoint = leftCodePoints.next();
    const rightPoint = rightCodePoints.next();
    if (leftPoint.done || rightPoint.done) return leftPoint.done ? -1 : 1;
    const difference = leftPoint.value.codePointAt(0) - rightPoint.value.codePointAt(0);
    if (difference !== 0) return difference;
  }
}

function ratio(count, total) {
  return total === 0 ? 0 : count / total;
}
