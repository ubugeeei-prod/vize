import fs from "node:fs";
import path from "node:path";

import type { LspChurnBudget } from "./churn-metrics.ts";
import type { ChurnCycleStats } from "./churn-percentiles.ts";

type LegacyChurnCycleStats = Pick<ChurnCycleStats, "cycles" | "minMs" | "medianMs" | "maxMs">;

/** The report fields `summary.md` renders; `metrics.json` serialises the whole payload. */
export type ChurnReportData = {
  status: string;
  fixture: string;
  fixtureRevision: string;
  publishes: number;
  peakRssKiB: number;
  peakProcessTreeRssKiB: number;
  budget: LspChurnBudget & { scale: number };
  cycleStats: (LegacyChurnCycleStats & Partial<ChurnCycleStats>) | null;
};

/**
 * Writes the churn `metrics.json` and `summary.md` artifacts that CI uploads
 * and appends to the step summary, mirroring the incremental-suite artifacts.
 */
export function writeChurnArtifacts(outputDir: string, title: string, data: ChurnReportData): void {
  fs.mkdirSync(outputDir, { recursive: true });
  fs.writeFileSync(path.join(outputDir, "metrics.json"), `${JSON.stringify(data, null, 2)}\n`);
  fs.writeFileSync(path.join(outputDir, "summary.md"), renderMarkdown(title, data));
}

function renderMarkdown(title: string, data: ChurnReportData): string {
  const stats = data.cycleStats;
  const budget = data.budget;
  const asMiB = (kib: number) => (kib > 0 ? `${(kib / 1024).toFixed(1)} MiB` : "unavailable");
  return [
    `## ${title}`,
    "",
    `Status: **${data.status}**. Fixture: \`${data.fixture}@${data.fixtureRevision}\`.`,
    "",
    stats == null ? "No churn cycles were measured." : renderCycleStats(stats, budget),
    "",
    `Diagnostics publishes observed: ${data.publishes}.`,
    "",
    `Peak server RSS: ${asMiB(data.peakRssKiB)} (budget ` +
      `${budget.maxPeakRssMiB * budget.scale} MiB). Peak process-tree RSS: ` +
      `${asMiB(data.peakProcessTreeRssKiB)} (budget ` +
      `${budget.maxPeakProcessTreeRssMiB * budget.scale} MiB, max ` +
      `${budget.maxProcessTreeSize} processes).`,
    "",
    `Ceilings are enforced at scale ${budget.scale} from the registry lspChurnBudget block; ` +
      "publish determinism, version ordering, stale-publish absence, and cancellation " +
      "convergence are hard assertions.",
    "",
  ].join("\n");
}

function renderCycleStats(
  stats: LegacyChurnCycleStats & Partial<ChurnCycleStats>,
  budget: ChurnReportData["budget"],
): string {
  const distribution =
    stats.p50Ms == null || stats.p95Ms == null || stats.p99Ms == null
      ? `median ${stats.medianMs.toFixed(1)} ms`
      : `p50 ${stats.p50Ms.toFixed(1)} ms, p95 ${stats.p95Ms.toFixed(1)} ms, ` +
        `p99 ${stats.p99Ms.toFixed(1)} ms`;
  return (
    `Churn cycles: ${stats.cycles} (broken -> repaired leaf and shared edits); min ` +
    `${stats.minMs.toFixed(1)} ms, ${distribution}, max ${stats.maxMs.toFixed(1)} ms ` +
    `(budget ${budget.budgetsMs.cycle * budget.scale} ms/cycle).`
  );
}
