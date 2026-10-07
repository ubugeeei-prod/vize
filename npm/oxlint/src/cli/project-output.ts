import type { OxlintProcessResult } from "./scoped-selection.ts";

export function unavailableProjectTransport(
  original: OxlintProcessResult,
  error: unknown,
): OxlintProcessResult {
  return {
    ...original,
    status: Math.max(original.status ?? 1, 1),
    stderr:
      original.stderr +
      `\nScript-safe Vue transport unavailable: ${error instanceof Error ? error.message : String(error)}\n`,
  };
}

export function projectOutputFormat(args: readonly string[]): string {
  let format = "default";
  for (let index = 0; index < args.length && args[index] !== "--"; index += 1) {
    if (args[index] === "-f" || args[index] === "--format") format = args[++index];
    else if (args[index].startsWith("--format=")) format = args[index].slice(9);
  }
  if (!["default", "json", "unix", "stylish"].includes(format))
    throw new Error(
      "Original-path project checks support default, JSON, Unix and Stylish reports.",
    );
  return format;
}

export function mergeProjectOutput(
  original: OxlintProcessResult,
  bridge: OxlintProcessResult,
  args: readonly string[],
): OxlintProcessResult {
  const format = projectOutputFormat(args);
  const unavailable = (phase: OxlintProcessResult): OxlintProcessResult => ({
    stdout: original.stdout,
    stderr: original.stderr + bridge.stderr + (phase === bridge ? bridge.stdout : ""),
    status: Math.max(original.status ?? 1, bridge.status ?? 1),
  });
  let stdout = original.stdout + bridge.stdout;
  if (format === "json") {
    for (const phase of [original, bridge]) {
      if (phase.stdout === "" && phase.status !== 0) return unavailable(phase);
      if (phase.stdout === "")
        throw new Error("Original-path project checks received an empty successful JSON report.");
    }
    const reports: Record<string, unknown>[] = [];
    for (const phase of [original, bridge]) {
      try {
        reports.push(JSON.parse(phase.stdout) as Record<string, unknown>);
      } catch (error) {
        if (phase.status !== 0) return unavailable(phase);
        throw error;
      }
    }
    const [source, template] = reports;
    if (
      !Array.isArray(source.diagnostics) ||
      !Array.isArray(template.diagnostics) ||
      typeof source.number_of_rules !== "number" ||
      typeof template.number_of_rules !== "number" ||
      typeof source.number_of_files !== "number" ||
      typeof source.threads_count !== "number" ||
      typeof template.threads_count !== "number" ||
      typeof source.start_time !== "number" ||
      typeof template.start_time !== "number"
    )
      throw new Error("Original-path project checks require complete Oxlint JSON reports.");
    const totals: Record<string, number> = {};
    for (const key of new Set([...Object.keys(source), ...Object.keys(template)])) {
      if (
        !/^number_of_|_count$/u.test(key) ||
        ["number_of_files", "number_of_rules", "threads_count"].includes(key)
      )
        continue;
      if (typeof source[key] !== "number" || typeof template[key] !== "number")
        throw new Error(
          `Original-path project checks cannot combine the Oxlint report total: ${key}`,
        );
      totals[key] = source[key] + template[key];
    }
    stdout = `${JSON.stringify({ ...source, ...totals, diagnostics: [...source.diagnostics, ...template.diagnostics], number_of_rules: source.number_of_rules + template.number_of_rules, threads_count: Math.max(source.threads_count, template.threads_count), start_time: source.start_time + template.start_time })}\n`;
  }
  return {
    stdout,
    stderr: original.stderr + bridge.stderr,
    status: Math.max(original.status ?? 1, bridge.status ?? 1),
  };
}
