import { createHash } from "node:crypto";
import { isAbsolute, relative } from "node:path";

/**
 * `documentedDifferences` is the reviewed ledger of expected vize-versus-vue-tsc
 * differences (tests/_fixtures/compat-documented-differences.json). An entry can
 * only ever cancel exactly one false positive against exactly one false negative
 * that share a file, severity, line and column: both tools must already report
 * something at that span, and both messages must match the ledger byte for byte
 * after whitespace normalization. A vize-only or vue-tsc-only diagnostic can
 * therefore never be absorbed, and a divergence that shifts position or wording
 * falls straight back into the false-positive/false-negative buckets.
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
  const baselineInput = collectVueTscDiagnostics(vueTscOutput, cwd);
  const vize = vizeInput.diagnostics;
  const baseline = baselineInput.diagnostics;
  const vizeGroups = groupByIdentity(vize);
  const baselineGroups = groupByIdentity(baseline);
  const identities = [...new Set([...vizeGroups.keys(), ...baselineGroups.keys()])].sort(byteOrder);
  const shared = [];
  const falsePositives = [];
  const falseNegatives = [];

  for (const identity of identities) {
    const candidates = vizeGroups.get(identity) ?? [];
    const expected = baselineGroups.get(identity) ?? [];
    const commonCount = Math.min(candidates.length, expected.length);
    for (let index = 0; index < commonCount; index += 1) {
      const candidate = candidates[index];
      shared.push({
        file: candidate.file,
        severity: candidate.severity,
        line: candidate.line,
        column: candidate.column,
        code: candidate.code,
        vizeMessage: candidate.message,
        baselineMessage: expected[index].message,
      });
    }
    falsePositives.push(...candidates.slice(commonCount));
    falseNegatives.push(...expected.slice(commonCount));
  }

  shared.sort(compareShared);
  falsePositives.sort(compareRecords);
  falseNegatives.sort(compareRecords);
  const documented = pairDocumentedDifferences(expectedDifferences, falsePositives, falseNegatives);
  const classified = { shared, falsePositives, falseNegatives, documentedDifferences: documented };
  const summary = {
    vizeDiagnosticCount: vize.length,
    baselineDiagnosticCount: baseline.length,
    sharedCount: shared.length,
    documentedDifferenceCount: documented.length,
    falsePositiveCount: falsePositives.length,
    falseNegativeCount: falseNegatives.length,
    falsePositiveRatio: ratio(falsePositives.length, vize.length),
    falseNegativeRatio: ratio(falseNegatives.length, baseline.length),
    vizeExcludedNonVueCount: vizeInput.excludedNonVueCount,
    baselineExcludedNonVueCount: baselineInput.excludedNonVueCount,
    baselineExcludedProjectCount: baselineInput.excludedProjectCount,
    baselineExcludedExternalCount: baselineInput.excludedExternalCount,
  };
  return {
    schema: "vize.fixtureTypecheckDivergence",
    version: 3,
    project: projectId,
    summary,
    ...classified,
    sha256: createHash("sha256")
      .update(JSON.stringify({ summary, ...classified }))
      .digest("hex"),
  };
}

function collectVizeDiagnostics(report, cwd) {
  if (report == null || typeof report !== "object" || !Array.isArray(report.files)) {
    invalid("Vize report must contain files");
  }
  const diagnostics = [];
  for (const [fileIndex, file] of report.files.entries()) {
    if (file == null || typeof file !== "object" || !Array.isArray(file.diagnostics)) {
      invalid(`Vize files[${fileIndex}] must contain diagnostics`);
    }
    const normalizedFile = normalizePath(file.file, cwd, `Vize files[${fileIndex}].file`);
    for (const [diagnosticIndex, diagnostic] of file.diagnostics.entries()) {
      if (typeof diagnostic !== "string") {
        invalid(`Vize diagnostic ${fileIndex}:${diagnosticIndex} must be a string`);
      }
      const match = /^(error|warning):(\d+):(\d+) \[TS(\d+)\] ([\s\S]+)$/.exec(diagnostic);
      if (match == null) invalid(`unparseable Vize diagnostic ${normalizedFile}`);
      diagnostics.push(record(normalizedFile, match[1], match[2], match[3], match[4], match[5]));
    }
  }
  return partitionVueDiagnostics(diagnostics);
}

function collectVueTscDiagnostics(output, cwd) {
  if (typeof output !== "string") invalid("vue-tsc output must be a string");
  const diagnostics = [];
  let excludedNonVueCount = 0;
  let excludedProjectCount = 0;
  let excludedExternalCount = 0;
  for (const line of output.replaceAll("\r\n", "\n").split("\n")) {
    const match = /^(.+)\((\d+),(\d+)\): (error|warning) TS(\d+): (.+)$/.exec(line);
    const projectMatch = /^(error|warning) TS(\d+): (.+)$/.exec(line);
    if (match != null) {
      const file = normalizeBaselinePath(match[1], cwd);
      if (file == null) excludedExternalCount += 1;
      else if (!file.endsWith(".vue")) excludedNonVueCount += 1;
      else diagnostics.push(record(file, match[4], match[2], match[3], match[5], match[6]));
    } else if (projectMatch != null) {
      record("<project>", projectMatch[1], "1", "1", projectMatch[2], projectMatch[3]);
      excludedProjectCount += 1;
    } else if (/\b(?:error|warning) TS\d+:/.test(line)) {
      invalid(`unparseable vue-tsc diagnostic: ${line}`);
    }
  }
  return { diagnostics, excludedNonVueCount, excludedProjectCount, excludedExternalCount };
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

function pairDocumentedDifferences(expected, falsePositives, falseNegatives) {
  const paired = [];
  for (const difference of expected) {
    const positiveIndex = findDocumented(falsePositives, difference, difference.vize);
    const negativeIndex = findDocumented(falseNegatives, difference, difference.baseline);
    if (positiveIndex < 0 || negativeIndex < 0) continue;
    falsePositives.splice(positiveIndex, 1);
    falseNegatives.splice(negativeIndex, 1);
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

function partitionVueDiagnostics(diagnostics) {
  const included = diagnostics.filter((diagnostic) => diagnostic.file.endsWith(".vue"));
  return {
    diagnostics: included,
    excludedNonVueCount: diagnostics.length - included.length,
  };
}

function normalizePath(value, cwd, label) {
  if (typeof value !== "string" || value.length === 0) invalid(`${label} must be non-empty`);
  let normalized = value.replaceAll("\\", "/");
  if (isAbsolute(normalized)) normalized = relative(cwd, normalized).replaceAll("\\", "/");
  if (normalized.startsWith("./")) normalized = normalized.slice(2);
  if (
    normalized.length === 0 ||
    isAbsolute(normalized) ||
    /^[A-Za-z]:\//.test(normalized) ||
    normalized.split("/").some((segment) => segment === "" || segment === "." || segment === "..")
  ) {
    invalid(`${label} must stay inside the fixture workspace`);
  }
  return normalized;
}

function normalizeBaselinePath(value, cwd) {
  if (typeof value !== "string" || value.length === 0) {
    invalid("vue-tsc diagnostic file must be non-empty");
  }
  let normalized = value.replaceAll("\\", "/");
  if (isAbsolute(normalized)) normalized = relative(cwd, normalized).replaceAll("\\", "/");
  if (normalized.startsWith("./")) normalized = normalized.slice(2);
  const segments = normalized.split("/");
  if (normalized.length === 0 || segments.some((segment) => segment === "" || segment === ".")) {
    invalid("vue-tsc diagnostic file must be normalized");
  }
  if (isAbsolute(normalized) || /^[A-Za-z]:\//.test(normalized) || segments.includes("..")) {
    return null;
  }
  return normalized;
}

function record(file, severity, line, column, code, message) {
  const values = [line, column, code].map(Number);
  if (values.some((value) => !Number.isSafeInteger(value) || value < 1)) {
    invalid(`diagnostic range and code must be positive safe integers: ${file}`);
  }
  const normalizedMessage = message.replace(/\s+/g, " ").trim();
  if (normalizedMessage.length === 0) invalid(`diagnostic message must be non-empty: ${file}`);
  return {
    file,
    severity,
    line: values[0],
    column: values[1],
    code: values[2],
    message: normalizedMessage,
  };
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

function invalid(message) {
  throw new Error(`Invalid typecheck divergence input: ${message}`);
}
