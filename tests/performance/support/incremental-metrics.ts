import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { inspect } from "node:util";
import { performance } from "node:perf_hooks";

import { repoRoot } from "../../_helpers/realworld-patch.ts";

export type IncrementalSuite = {
  /** Metrics directory name under `target/vize-tests/metrics/`. */
  id: string;
  /** Markdown summary heading, e.g. `Misskey LSP Incremental Oracle`. */
  title: string;
};

export function incrementalMetricsDir(suiteId: string): string {
  return path.join(repoRoot, "target/vize-tests/metrics", suiteId);
}

type MetricContext = {
  fixture: string;
  revision: string;
  vueFiles: number;
  sourceFiles: number;
  baselineDiagnostics: number;
};

export class IncrementalMetrics {
  private readonly timingsMs: Record<string, number> = {};
  private readonly rssSamplesKiB: Record<string, number> = {};
  private readonly processId: number;
  private readonly suite: IncrementalSuite;

  constructor(processId: number, suite: IncrementalSuite) {
    this.processId = processId;
    this.suite = suite;
  }

  async measure<T>(name: string, operation: () => Promise<T>): Promise<T> {
    const startedAt = performance.now();
    try {
      return await operation();
    } finally {
      this.timingsMs[name] = performance.now() - startedAt;
      this.sampleRss(name);
    }
  }

  sampleRss(name: string): void {
    const rss = processRssKiB(this.processId);
    if (rss != null) this.rssSamplesKiB[name] = rss;
  }

  write(context: MetricContext, failure?: unknown): void {
    const outputDir = incrementalMetricsDir(this.suite.id);
    fs.mkdirSync(outputDir, { recursive: true });
    const sampledPeakRssKiB = Math.max(0, ...Object.values(this.rssSamplesKiB));
    const data = {
      schemaVersion: 1,
      status: failure == null ? "passed" : "failed",
      failure:
        failure instanceof Error ? failure.message : failure == null ? null : inspect(failure),
      commit: gitHead(),
      fixture: context.fixture,
      fixtureRevision: context.revision,
      corpus: {
        vueFiles: context.vueFiles,
        vueAndTypeScriptFiles: context.sourceFiles,
        baselineDiagnostics: context.baselineDiagnostics,
      },
      runtime: {
        platform: process.platform,
        architecture: process.arch,
        node: process.version,
        cpuCount: os.cpus().length,
        cpuModel: os.cpus()[0]?.model ?? "unknown",
      },
      timingsMs: this.timingsMs,
      rssSamplesKiB: this.rssSamplesKiB,
      sampledPeakRssKiB,
      note: "Latency and RSS are report-only; diagnostic, completion, hover, and repair oracles are gated.",
    };
    fs.writeFileSync(path.join(outputDir, "metrics.json"), `${JSON.stringify(data, null, 2)}\n`);
    fs.writeFileSync(path.join(outputDir, "summary.md"), renderMarkdown(this.suite.title, data));
  }
}

export function countFiles(
  root: string,
  extensions: ReadonlySet<string>,
  ignoreDirectories?: ReadonlySet<string>,
): number {
  let count = 0;
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const filePath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      if (ignoreDirectories?.has(entry.name)) continue;
      count += countFiles(filePath, extensions, ignoreDirectories);
    } else if (extensions.has(path.extname(entry.name))) {
      count += 1;
    }
  }
  return count;
}

function processRssKiB(processId: number): number | null {
  if (process.platform === "win32") return null;
  const result = spawnSync("ps", ["-o", "rss=", "-p", String(processId)], { encoding: "utf8" });
  if (result.status !== 0) return null;
  const value = Number.parseInt(result.stdout.trim(), 10);
  return Number.isFinite(value) && value > 0 ? value : null;
}

function gitHead(): string {
  const result = spawnSync("git", ["rev-parse", "HEAD"], { cwd: repoRoot, encoding: "utf8" });
  return result.status === 0 ? result.stdout.trim() : "unknown";
}

function renderMarkdown(
  title: string,
  data: {
    status: string;
    fixture: string;
    fixtureRevision: string;
    corpus: { vueFiles: number; vueAndTypeScriptFiles: number; baselineDiagnostics: number };
    timingsMs: Record<string, number>;
    sampledPeakRssKiB: number;
  },
): string {
  const lines = [
    `## ${title}`,
    "",
    `Status: **${data.status}**. Fixture: \`${data.fixture}@${data.fixtureRevision}\`.`,
    "",
    `Corpus: ${data.corpus.vueFiles} Vue files; ${data.corpus.vueAndTypeScriptFiles} Vue/TS files; ${data.corpus.baselineDiagnostics} baseline diagnostics.`,
    "",
    "| Stage | Time |",
    "| --- | ---: |",
  ];
  for (const [stage, milliseconds] of Object.entries(data.timingsMs)) {
    lines.push(`| ${stage} | ${milliseconds.toFixed(1)} ms |`);
  }
  lines.push(
    "",
    `Sampled peak LSP RSS: ${data.sampledPeakRssKiB > 0 ? `${(data.sampledPeakRssKiB / 1024).toFixed(1)} MiB` : "unavailable"}.`,
    "",
    "Timing and RSS are report-only. The clean/broken/repaired diagnostics, completion, hover, and dependency propagation are hard assertions.",
    "",
  );
  return lines.join("\n");
}
