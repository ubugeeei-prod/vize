import type { OxlintHtmlCompleted } from "@vizejs/native";
import type { OxlintProcessResult } from "./scoped-selection.ts";
import type { HtmlContext } from "./html-operation.ts";
import { joinProjectJson } from "./project-json.ts";

const noFiles = "No files found to lint. Please check your paths and ignore patterns.\n";
const stylishFooter = new RegExp(
  String.raw`\n(?:\u001B\[(?:31|33)m)?✖ [1-9][0-9]* problems? \([0-9]+ errors?, [0-9]+ warnings?\)(?:\u001B\[0m)?\n$`,
  "u",
);

function completeJson(text: string): Record<string, unknown> {
  const report: unknown = JSON.parse(text);
  if (typeof report !== "object" || report == null || Array.isArray(report))
    throw new Error("HTML join requires an actual complete host JSON object");
  const result = report as Record<string, unknown>;
  if (
    !Array.isArray(result.diagnostics) ||
    !["number_of_files", "number_of_rules", "threads_count", "start_time"].every(
      (key) => typeof result[key] === "number" && Number.isFinite(result[key]) && result[key] >= 0,
    )
  )
    throw new Error("HTML join requires genuine host file/rule/thread/time authority");
  return result;
}

/** Genuine child packets establish host setup; no additional provider query. */
export function qualifyHtmlHost(context: HtmlContext, original: OxlintProcessResult): void {
  if (context.failure || !context.options || !context.provider)
    throw new Error(context.failure ?? "original HTML host context is unavailable");
  const phases = original.phases ?? (original.observation ? [original.observation] : []);
  if (phases.length === 0)
    throw new Error("original HTML requires complete actual host observations");
  for (const phase of phases) {
    if (
      phase.signal ||
      phase.error ||
      ![0, 1].includes(phase.status ?? -1) ||
      phase.cwd !== context.options.cwd ||
      (phase.args[0] !== context.provider.entrypoint &&
        phase.args[0] !== context.provider.requestedEntrypoint)
    )
      throw new Error("actual host phase failed before qualified HTML execution");
    if (
      Buffer.from(phase.stderrBytes).length !== 0 ||
      Object.keys(context.environment).some(
        (key) => phase.environment[key] !== context.environment[key],
      )
    )
      throw new Error("actual host phase lost its original presentation/setup context");
    const rawOutput = Buffer.from(phase.stdoutBytes);
    const output = rawOutput.toString("utf8");
    if (!rawOutput.equals(Buffer.from(output)))
      throw new Error("actual host output is not lossless UTF8");
    if (output.startsWith(noFiles)) continue; // Native zero-count and the closed packet gate decide exclusion.
    const report = output;
    if (context.options.format === "json") completeJson(report);
    else if (
      context.options.format === "default" &&
      !/Finished in [0-9]+(?:\.[0-9]+)?(?:ms|s) on [0-9]+ files? with [0-9]+ rules? using [0-9]+ threads?\.\n$/u.test(
        report,
      )
    )
      throw new Error("actual default host report is incomplete");
    else if (
      context.options.format === "unix" &&
      !(phase.status === 0 && report === "") &&
      !/\n[1-9][0-9]* problems?\n$/u.test(report)
    )
      throw new Error("actual Unix host report is incomplete");
    else if (
      context.options.format === "stylish" &&
      !(phase.status === 0 && report === "") &&
      !stylishFooter.test(report)
    )
      throw new Error("actual Stylish host report is incomplete");
  }
}

/** Recognize the pinned closed normal walker-empty packet, never a substring. */
function eligibleEmptyHost(
  original: OxlintProcessResult,
  native: OxlintHtmlCompleted,
  args: readonly string[],
): string | undefined {
  if (
    native.rootDecision !== "Eligible" ||
    original.stderr !== "" ||
    original.observation?.signal ||
    original.observation?.error ||
    original.status !== (args.includes("--no-error-on-unmatched-pattern") ? 0 : 1) ||
    !original.stdout.startsWith(noFiles)
  )
    return undefined;
  const rest = original.stdout.slice(noFiles.length);
  if (native.format === "unix" || native.format === "stylish")
    return rest === "" ? rest : undefined;
  if (native.format === "default")
    return /^Finished in [0-9]+(?:\.[0-9]+)?(?:ms|s) on 0 files with [0-9]+ rules? using [0-9]+ threads?\.\n$/u.test(
      rest,
    )
      ? rest
      : undefined;
  try {
    const report = completeJson(rest);
    if (
      (report.diagnostics as unknown[]).length !== 0 ||
      report.number_of_files !== 0 ||
      Object.keys(report).length !== 5
    )
      return undefined;
    // joinProjectJson's unique-property/layout scanner also qualifies the closed document.
    return joinProjectJson(rest, '{"diagnostics":[]}', {}) === rest ? rest : undefined;
  } catch {
    return undefined;
  }
}

export function mergeHtmlOutput(
  original: OxlintProcessResult,
  native: OxlintHtmlCompleted,
  args: readonly string[],
): OxlintProcessResult {
  if (native.executedFileCount === 0) return original;
  if (original.observation?.signal || original.observation?.error || original.status == null)
    throw new Error("original host failed before a qualified HTML join");
  const empty = eligibleEmptyHost(original, native, args);
  if (original.stdout.startsWith(noFiles) && empty == null)
    throw new Error("original NoFilesFound packet is not the qualified normal walker-empty branch");
  let stdout = empty ?? original.stdout;
  if (native.format === "json") {
    const report = completeJson(stdout);
    const rows: unknown = JSON.parse(native.jsonDiagnostics);
    if (!Array.isArray(rows))
      throw new Error("native HTML did not render complete diagnostic fragments");
    stdout = joinProjectJson(stdout, `{"diagnostics":${native.jsonDiagnostics}}`, {
      number_of_files: Number(report.number_of_files) + native.executedFileCount,
      start_time: Number(report.start_time) + native.elapsedSeconds,
    });
  } else stdout += native.output;
  const nativeFailure =
    native.errors > 0 ||
    ((args.includes("--deny-warnings") || native.projection.denyWarnings) && native.warnings > 0);
  return {
    ...original,
    stdout,
    rawStdout: undefined,
    status: Math.max(empty == null ? original.status : 0, nativeFailure ? 1 : 0),
  };
}

export function unavailableHtml(
  original: OxlintProcessResult,
  error: unknown,
): OxlintProcessResult {
  const suffix = `\nOriginal HTML unavailable: ${error instanceof Error ? error.message : String(error)}\n`;
  return {
    ...original,
    rawStderr: Buffer.concat([
      original.rawStderr ?? Buffer.from(original.stderr),
      Buffer.from(suffix),
    ]),
    status: Math.max(original.status ?? 1, 1),
    stderr: original.stderr + suffix,
  };
}
