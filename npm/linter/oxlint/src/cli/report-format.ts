import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export type ConvertedReportFormat = "github" | "gitlab";

interface Span {
  readonly offset: number;
  readonly length: number;
  readonly line: number;
  readonly column: number;
}

interface Diagnostic {
  readonly message: string;
  readonly code?: string;
  readonly severity: string;
  readonly filename?: string;
  readonly labels?: ReadonlyArray<{ readonly span: Span }>;
}

interface Report {
  readonly diagnostics: Diagnostic[];
  readonly number_of_files: number;
  readonly number_of_rules: number;
  readonly threads_count: number;
  readonly start_time: number;
}

/** The last requested wrapper-owned format before `--`, when conversion is required. */
export function requestedConvertedFormat(
  args: readonly string[],
): ConvertedReportFormat | undefined {
  let format: string | undefined;
  for (let index = 0; index < args.length && args[index] !== "--"; index += 1) {
    const arg = args[index];
    if (arg === "-f" || arg === "--format") format = args[++index];
    else if (arg.startsWith("--format=")) format = arg.slice("--format=".length);
  }
  return format === "github" || format === "gitlab" ? format : undefined;
}

/** Preserve all CLI spelling and ordering while making the internal report mergeable. */
export function withJsonReport(args: readonly string[]): string[] {
  const result = [...args];
  for (let index = 0; index < result.length && result[index] !== "--"; index += 1) {
    const arg = result[index];
    if (arg === "-f" || arg === "--format") {
      if (result[index + 1] != null) result[index + 1] = "json";
      index += 1;
    } else if (arg.startsWith("--format=")) {
      result[index] = "--format=json";
    }
  }
  return result;
}

export function renderConvertedReport(
  stdout: string,
  format: ConvertedReportFormat,
  cwd: string,
): string {
  const report = parseReport(stdout);
  return format === "github" ? renderGitHub(report, cwd) : renderGitLab(report, cwd);
}

function parseReport(stdout: string): Report {
  const value = JSON.parse(stdout) as Partial<Report> | null;
  if (
    value == null ||
    !Array.isArray(value.diagnostics) ||
    !finite(value.number_of_files) ||
    !finite(value.number_of_rules) ||
    !finite(value.threads_count) ||
    !finite(value.start_time)
  ) {
    throw new Error("GitHub and GitLab output require a complete Oxlint JSON report.");
  }
  for (const diagnostic of value.diagnostics) {
    if (
      diagnostic == null ||
      typeof diagnostic !== "object" ||
      typeof diagnostic.message !== "string" ||
      typeof diagnostic.severity !== "string"
    ) {
      throw new Error("GitHub and GitLab output received an incomplete Oxlint diagnostic.");
    }
  }
  return value as Report;
}

function renderGitHub(report: Report, cwd: string): string {
  const diagnostics = report.diagnostics.map((diagnostic) => githubAnnotation(diagnostic, cwd));
  const errors = report.diagnostics.filter((diagnostic) => diagnostic.severity === "error").length;
  const warnings = report.diagnostics.length - errors;
  const duration =
    report.start_time < 1
      ? `${Math.floor(report.start_time * 1_000)}ms`
      : `${report.start_time.toFixed(1)}s`;
  const summary = [
    `Found ${warnings} warning${warnings === 1 ? "" : "s"} and ${errors} error${errors === 1 ? "" : "s"}.`,
    `Finished in ${duration} on ${report.number_of_files} file${report.number_of_files === 1 ? "" : "s"} with ${report.number_of_rules} rules using ${report.threads_count} threads.`,
  ].join("\n");
  return `${diagnostics.length > 0 ? `${diagnostics.join("\n")}\n\n` : ""}${summary}\n`;
}

function githubAnnotation(diagnostic: Diagnostic, cwd: string): string {
  const span = primarySpan(diagnostic);
  const filename = diagnostic.filename == null ? undefined : displayPath(diagnostic.filename, cwd);
  const end =
    span == null || diagnostic.filename == null
      ? undefined
      : endPosition(diagnostic.filename, span, cwd);
  const properties: Array<[string, string | number]> = [];
  if (filename != null) properties.push(["file", filename]);
  if (span != null) {
    properties.push(["line", span.line], ["endLine", end?.line ?? span.line]);
    properties.push(["col", span.column], ["endColumn", end?.column ?? span.column]);
  }
  properties.push(["title", diagnostic.code ?? "oxlint"]);
  const encoded = properties
    .map(([name, value]) => `${name}=${escapeCommandProperty(String(value))}`)
    .join(",");
  const location =
    filename != null && span != null ? `${filename}:${span.line}:${span.column}: ` : "";
  return `::${diagnostic.severity === "error" ? "error" : "warning"} ${encoded}::${escapeCommandData(`${location}${diagnostic.message}`)}`;
}

function renderGitLab(report: Report, cwd: string): string {
  const rows = report.diagnostics.map((diagnostic) => {
    const span = primarySpan(diagnostic);
    const filename = diagnostic.filename == null ? "" : displayPath(diagnostic.filename, cwd);
    const end =
      span == null || diagnostic.filename == null
        ? undefined
        : endPosition(diagnostic.filename, span, cwd);
    const identity = JSON.stringify([
      filename,
      diagnostic.code ?? "oxlint",
      diagnostic.message,
      span?.line ?? 1,
      end?.line ?? span?.line ?? 1,
      diagnostic.severity,
    ]);
    return {
      description: diagnostic.message,
      check_name: diagnostic.code ?? "oxlint",
      fingerprint: createHash("sha256").update(identity).digest("hex").slice(0, 16),
      severity:
        diagnostic.severity === "error"
          ? "critical"
          : diagnostic.severity === "warning"
            ? "major"
            : "minor",
      location: {
        path: filename,
        lines: { begin: span?.line ?? 1, end: end?.line ?? span?.line ?? 1 },
      },
    };
  });
  return `${JSON.stringify(rows, null, 2)}\n`;
}

function primarySpan(diagnostic: Diagnostic): Span | undefined {
  const span = diagnostic.labels?.[0]?.span;
  return span != null &&
    [span.offset, span.length, span.line, span.column].every(Number.isSafeInteger)
    ? span
    : undefined;
}

function endPosition(filename: string, span: Span, cwd: string): { line: number; column: number } {
  try {
    const bytes = fs.readFileSync(path.resolve(cwd, filename));
    const endOffset = Math.min(span.offset + span.length, bytes.length);
    const before = bytes.subarray(0, endOffset).toString("utf8");
    if (!Buffer.from(before, "utf8").equals(bytes.subarray(0, endOffset)))
      return { line: span.line, column: span.column };
    const lines = before.split("\n");
    return { line: lines.length, column: (lines.at(-1)?.length ?? 0) + 1 };
  } catch {
    return { line: span.line, column: span.column };
  }
}

function displayPath(filename: string, cwd: string): string {
  const absolute = path.resolve(cwd, filename);
  const root = repositoryRoot(cwd);
  const relative = path.relative(root, absolute);
  return relative !== "" && !relative.startsWith(`..${path.sep}`) && relative !== ".."
    ? relative.split(path.sep).join("/")
    : filename.split(path.sep).join("/");
}

function repositoryRoot(cwd: string): string {
  let current = path.resolve(cwd);
  while (true) {
    if (fs.existsSync(path.join(current, ".git"))) return current;
    const parent = path.dirname(current);
    if (parent === current) return path.resolve(cwd);
    current = parent;
  }
}

function escapeCommandProperty(value: string): string {
  return escapeCommandData(value).replaceAll(":", "%3A").replaceAll(",", "%2C");
}

function escapeCommandData(value: string): string {
  return value.replaceAll("%", "%25").replaceAll("\r", "%0D").replaceAll("\n", "%0A");
}

function finite(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}
