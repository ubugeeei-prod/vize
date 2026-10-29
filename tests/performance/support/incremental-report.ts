import fs from "node:fs";
import path from "node:path";

import type { LspIncrementalBudget } from "./incremental-metrics.ts";
import type { PeakProcessTreeRssSample } from "./process-metrics.ts";

export type IncrementalReportData = {
  schemaVersion: 3;
  status: string;
  fixture: string;
  fixtureRevision: string;
  corpus: { vueFiles: number; vueAndTypeScriptFiles: number; baselineDiagnostics: number };
  budget: LspIncrementalBudget & { scale: number };
  timingsMs: Record<string, number>;
  processTreeAttributionSetupMs: number;
  processTreeAttributionMs: Record<string, number>;
  sampledPeakRssKiB: number;
  sampledPeakProcessTreeRssKiB: number;
  sampledPeakProcessTreeSize: number;
  sampledPeakProcessTree: PeakProcessTreeRssSample | null;
};

export function writeIncrementalArtifacts(
  outputDir: string,
  title: string,
  data: IncrementalReportData,
): void {
  fs.mkdirSync(outputDir, { recursive: true });
  fs.writeFileSync(path.join(outputDir, "metrics.json"), `${JSON.stringify(data, null, 2)}\n`);
  fs.writeFileSync(path.join(outputDir, "summary.md"), renderMarkdown(title, data));
}

function renderMarkdown(title: string, data: IncrementalReportData): string {
  const scale = data.budget.scale;
  const asMiB = (kib: number) => (kib > 0 ? `${(kib / 1024).toFixed(1)} MiB` : "unavailable");
  const lines = [
    `## ${markdownCell(title)}`,
    "",
    `Status: **${markdownCell(data.status)}**. Fixture: ` +
      `${markdownCell(data.fixture)}@${markdownCell(data.fixtureRevision)}.`,
    "",
    `Corpus: ${data.corpus.vueFiles} Vue files; ${data.corpus.vueAndTypeScriptFiles} Vue/TS files; ${data.corpus.baselineDiagnostics} baseline diagnostics.`,
    "",
    "| Stage | Time | Budget |",
    "| --- | ---: | ---: |",
  ];
  for (const [stage, milliseconds] of Object.entries(data.timingsMs)) {
    const budgetMs = data.budget.laneBudgetsMs[stage];
    const budgetCell = budgetMs == null ? "-" : `${budgetMs * scale} ms`;
    lines.push(`| ${markdownCell(stage)} | ${milliseconds.toFixed(1)} ms | ${budgetCell} |`);
  }
  lines.push(
    "",
    `Stable process attribution ran outside lane timings: setup ` +
      `${data.processTreeAttributionSetupMs.toFixed(1)} ms; max sample ${Math.max(
        0,
        ...Object.values(data.processTreeAttributionMs),
      ).toFixed(1)} ms; each ambiguous snapshot is bounded to three immediate attempts.`,
    "",
    `Sampled peak LSP RSS: ${asMiB(data.sampledPeakRssKiB)} ` +
      `(budget ${data.budget.maxPeakRssMiB * scale} MiB).`,
    `Sampled peak process-tree RSS: ${asMiB(data.sampledPeakProcessTreeRssKiB)} ` +
      `(budget ${data.budget.maxPeakProcessTreeRssMiB * scale} MiB, max ` +
      `${data.budget.maxProcessTreeSize} processes; max observed ` +
      `${data.sampledPeakProcessTreeSize}).`,
  );
  renderPeakAttribution(lines, data.sampledPeakProcessTree, asMiB);
  lines.push(
    "",
    `Latency, RSS, process-tree, and hang ceilings are enforced at scale ${scale} from the registry ` +
      "lspIncrementalBudget block. The clean/broken/repaired diagnostics, completion, hover, " +
      "and dependency propagation are hard assertions.",
    "",
  );
  return lines.join("\n");
}

function renderPeakAttribution(
  lines: string[],
  peak: PeakProcessTreeRssSample | null,
  asMiB: (kib: number) => string,
): void {
  if (peak == null) return;
  lines.push(
    "",
    `Peak attribution: ${markdownCell(peak.label)}; ${peak.processes} members; exact member ` +
      `sum ${asMiB(peak.memberRssSumKiB)}.`,
    "",
    "| Tree role | Runtime role | PID | PPID | RSS | Birth ticks | Executable kind | Executable SHA-256 | argc | argv SHA-256 |",
    "| --- | --- | ---: | ---: | ---: | ---: | --- | --- | ---: | --- |",
  );
  for (const member of peak.members) {
    const identity = member.identity;
    lines.push(
      `| ${member.role} | ${identity?.runtimeRole ?? "unavailable"} | ${member.pid} | ` +
        `${member.parentPid} | ${asMiB(member.rssKiB)} | ` +
        `${identity?.birthTicks ?? "unavailable"} | ` +
        `${identity?.executableKind ?? "unavailable"} | ` +
        `${identity?.executableSha256 ?? "unavailable"} | ` +
        `${identity?.argvCount ?? "unavailable"} | ` +
        `${identity?.argvSha256 ?? "unavailable"} |`,
    );
  }
}

function markdownCell(value: string): string {
  return value
    .replaceAll("\n", " ")
    .replaceAll("\r", " ")
    .replace(/[\\|`*_[\]{}()#+!<>]/gu, "\\$&");
}
