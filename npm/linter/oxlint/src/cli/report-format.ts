import { createHash } from "node:crypto";

/**
 * Formats Oxlint can print natively but that the original-path Vue pipeline
 * cannot merge across its phases. They are produced by running the whole
 * pipeline as `json` and converting the merged report once at the end.
 */
export type ConvertedReportFormat = "gitlab" | "github";

const CONVERTED_FORMATS: readonly string[] = ["gitlab", "github"];

interface FormatFlag {
  /** Index of the flag token itself. */
  index: number;
  /** `true` for the `--format=value` spelling, which has no separate value token. */
  inline: boolean;
  value: string;
}

function findFormatFlag(args: readonly string[]): FormatFlag | undefined {
  let found: FormatFlag | undefined;
  for (let index = 0; index < args.length && args[index] !== "--"; index += 1) {
    const arg = args[index];
    if (arg === "-f" || arg === "--format") {
      if (index + 1 < args.length) found = { index, inline: false, value: args[index + 1] };
      index += 1;
    } else if (arg.startsWith("--format=")) {
      found = { index, inline: true, value: arg.slice("--format=".length) };
    }
  }
  return found;
}

export function requestedConvertedFormat(
  args: readonly string[],
): ConvertedReportFormat | undefined {
  const flag = findFormatFlag(args);
  return flag && CONVERTED_FORMATS.includes(flag.value)
    ? (flag.value as ConvertedReportFormat)
    : undefined;
}

/** Returns `args` with the requested format replaced by `json`. */
export function withJsonFormat(args: readonly string[]): string[] {
  const flag = findFormatFlag(args);
  const next = [...args];
  if (!flag) return next;
  if (flag.inline) next[flag.index] = "--format=json";
  else next[flag.index + 1] = "json";
  return next;
}

interface JsonDiagnostic {
  message?: unknown;
  code?: unknown;
  severity?: unknown;
  filename?: unknown;
  labels?: unknown;
}

interface Position {
  line: number;
  column: number;
}

function diagnosticsOf(report: string): JsonDiagnostic[] {
  const parsed: unknown = JSON.parse(report);
  const diagnostics = (parsed as { diagnostics?: unknown } | null)?.diagnostics;
  if (!Array.isArray(diagnostics))
    throw new Error("Oxlint JSON report has no diagnostics array to convert.");
  return diagnostics as JsonDiagnostic[];
}

function startOf(diagnostic: JsonDiagnostic): Position {
  const labels = Array.isArray(diagnostic.labels) ? diagnostic.labels : [];
  const span = (labels[0] as { span?: Partial<Position> } | undefined)?.span;
  return { line: span?.line ?? 1, column: span?.column ?? 1 };
}

function text(value: unknown, fallback = ""): string {
  return typeof value === "string" ? value : fallback;
}

function gitlabSeverity(severity: string): string {
  return severity === "error" ? "critical" : severity === "warning" ? "major" : "minor";
}

export function renderGitlab(report: string): string {
  const entries = diagnosticsOf(report).map((diagnostic) => {
    const path = text(diagnostic.filename);
    const check = text(diagnostic.code);
    const message = text(diagnostic.message).split("\n")[0];
    const { line } = startOf(diagnostic);
    const fingerprint = createHash("sha256")
      .update([path, check, message, line].join("\0"))
      .digest("hex")
      .slice(0, 16);
    return {
      description: message,
      check_name: check,
      fingerprint,
      severity: gitlabSeverity(text(diagnostic.severity)),
      location: { path, lines: { begin: line, end: line } },
    };
  });
  return `${JSON.stringify(entries, null, 2)}\n`;
}

function escapeProperty(value: string): string {
  return value
    .replace(/%/gu, "%25")
    .replace(/\r/gu, "%0D")
    .replace(/\n/gu, "%0A")
    .replace(/:/gu, "%3A")
    .replace(/,/gu, "%2C");
}

function escapeData(value: string): string {
  return value.replace(/%/gu, "%25").replace(/\r/gu, "%0D").replace(/\n/gu, "%0A");
}

export function renderGithub(report: string): string {
  const lines = diagnosticsOf(report).map((diagnostic) => {
    const path = text(diagnostic.filename);
    const { line, column } = startOf(diagnostic);
    const level = text(diagnostic.severity) === "error" ? "error" : "warning";
    const props = `file=${escapeProperty(path)},line=${line},col=${column},title=${escapeProperty(text(diagnostic.code))}`;
    const message = text(diagnostic.message).split("\n")[0];
    return `::${level} ${props}::${escapeData(`${path}:${line}:${column}: ${message}`)}`;
  });
  return lines.length === 0 ? "" : `${lines.join("\n")}\n`;
}

export function renderConvertedReport(report: string, format: ConvertedReportFormat): string {
  return format === "gitlab" ? renderGitlab(report) : renderGithub(report);
}

/**
 * Converts a merged JSON report. Anything that is not a JSON report (for
 * example Oxlint's plain "No files found" notice, or an error packet) is
 * returned untouched so failures stay visible instead of being rewritten.
 */
export function renderConvertedReportOrPassThrough(
  stdout: string,
  format: ConvertedReportFormat,
): string {
  let parsed: unknown;
  try {
    parsed = JSON.parse(stdout);
  } catch {
    return stdout;
  }
  if (!Array.isArray((parsed as { diagnostics?: unknown } | null)?.diagnostics)) return stdout;
  return renderConvertedReport(stdout, format);
}
