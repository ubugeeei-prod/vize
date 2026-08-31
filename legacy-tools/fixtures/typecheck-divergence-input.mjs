/**
 * Parsing and normalization of the two diagnostic streams the divergence
 * comparator classifies: `vize check --format json` on one side, the plain
 * `vue-tsc --pretty false` text output on the other.
 *
 * Both sides are reduced to the same record shape — file relative to the
 * workspace, severity, 1-based line and column, numeric TypeScript code, and a
 * whitespace-normalized message — and anything that cannot be parsed is a hard
 * error rather than a silently dropped diagnostic.
 */
import { isAbsolute, relative } from "node:path";

export function invalid(message) {
  throw new Error(`Invalid typecheck divergence input: ${message}`);
}

export function collectVizeDiagnostics(report, cwd) {
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

export function collectVueTscDiagnostics(output, cwd, options = {}) {
  if (typeof output !== "string") invalid("vue-tsc output must be a string");
  const diagnostics = [];
  let excludedNonVueCount = 0;
  let excludedProjectCount = 0;
  let excludedExternalCount = 0;
  let excludedSupportVueCount = 0;
  const comparableVueFiles = options.comparableVueFiles;
  for (const line of output.replaceAll("\r\n", "\n").split("\n")) {
    const match = /^(.+)\((\d+),(\d+)\): (error|warning) TS(\d+): (.+)$/.exec(line);
    const projectMatch = /^(error|warning) TS(\d+): (.+)$/.exec(line);
    if (match != null) {
      const file = normalizeBaselinePath(match[1], cwd);
      if (file == null) excludedExternalCount += 1;
      else if (!file.endsWith(".vue")) excludedNonVueCount += 1;
      else if (
        comparableVueFiles instanceof Set &&
        !comparableVueFiles.has(file) &&
        isSupportVueFile(file)
      ) {
        excludedSupportVueCount += 1;
      } else diagnostics.push(record(file, match[4], match[2], match[3], match[5], match[6]));
    } else if (projectMatch != null) {
      // Validated for shape only: a project-level diagnostic has no file or
      // position to compare, so the normalized record is discarded and the line
      // is counted as excluded.
      record("<project>", projectMatch[1], "1", "1", projectMatch[2], projectMatch[3]);
      excludedProjectCount += 1;
    } else if (/\b(?:error|warning) TS\d+:/.test(line)) {
      invalid(`unparseable vue-tsc diagnostic: ${line}`);
    }
  }
  return {
    diagnostics,
    excludedNonVueCount,
    excludedProjectCount,
    excludedExternalCount,
    excludedSupportVueCount,
  };
}

export function normalizePath(value, cwd, label) {
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

function partitionVueDiagnostics(diagnostics) {
  const included = diagnostics.filter((diagnostic) => diagnostic.file.endsWith(".vue"));
  return {
    diagnostics: included,
    excludedNonVueCount: diagnostics.length - included.length,
  };
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

/**
 * Support SFCs live under a dot-directory such as `docs/.vitepress`: they enter
 * the vue-tsc program through the composite graph, but they are never part of
 * the compared Vize corpus. The diagnostic collector and the coverage collector
 * must agree on that rule, so this predicate is the single classifier both use.
 */
export function isSupportVueFile(file) {
  // Only a dot *directory* makes an SFC support: a dot-prefixed filename such as
  // `.Local.vue` is authored fixture source, and treating it as support would
  // silently drop it from the compared corpus.
  const segments = file.split("/");
  return segments.slice(0, -1).some((segment) => segment.startsWith("."));
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
