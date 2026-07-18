import { createHash } from "node:crypto";
import { isAbsolute, relative } from "node:path";

export function compareTypecheckDiagnostics({ projectId, cwd, vizeReport, vueTscOutput }) {
  if (typeof projectId !== "string" || projectId.length === 0) invalid("project id is required");
  if (typeof cwd !== "string" || !isAbsolute(cwd)) invalid("cwd must be absolute");
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
  const classified = { shared, falsePositives, falseNegatives };
  const summary = {
    vizeDiagnosticCount: vize.length,
    baselineDiagnosticCount: baseline.length,
    sharedCount: shared.length,
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
    version: 2,
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
