/** Report rendering is separate from measurement and never changes admission. */
import { existsSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { arch, cpus, release } from "node:os";
import { join } from "node:path";
import { sha256, writeJson } from "./pr-performance-observation-evidence.ts";
import type { binaryEvidence, corpusInventory } from "./pr-performance-observation-evidence.ts";
import type { BenchmarkResult } from "./compare-pr-results.ts";
import type { Lane, Options } from "./pr-performance-observation.ts";

export function writeObservationReport(context: {
  options: Options;
  corpus: ReturnType<typeof corpusInventory>;
  after: ReturnType<typeof corpusInventory>;
  unchanged: boolean;
  binaries: { base: ReturnType<typeof binaryEvidence>; head: ReturnType<typeof binaryEvidence> };
  backend: ReturnType<typeof binaryEvidence> | null;
  vue: { path: string; packageJsonSha256: string; version: unknown } | null;
  results: Lane[];
}) {
  const { options, corpus, after, unchanged, binaries, backend, vue, results } = context;
  const { out } = options;
  const report = {
    schemaVersion: 1,
    advisory: true,
    failOnRegression: false,
    source: { base: options.baseSource, head: options.headSource },
    sourceBuildReceipts: options.buildReceipts
      ? Object.fromEntries(
          (statSync(options.buildReceipts).isDirectory() ? ["base.json", "head.json"] : [""]).map(
            (name) => {
              const file = name ? join(options.buildReceipts!, name) : options.buildReceipts!;
              return [
                name || "manifest",
                {
                  path: file,
                  sha256: sha256(file),
                  contents: JSON.parse(readFileSync(file, "utf8")),
                },
              ];
            },
          ),
        )
      : null,
    methodology: {
      estimator:
        "existing compare-pr paired head/base median ratios; alternating sequential fresh processes",
      runs: options.runs,
      warmups: options.warmups,
      processFresh: true,
      coldQualified: false,
      osPageCache: "warmed by warmups; never flushed",
      persistentCaches: "may exist; not independently excluded",
      latencyIncludes:
        "spawn and bounded timeout supervisor when available; no concurrent base/head samples",
      rss: "separate one-sample GNU time max RSS in bytes; process/waited-descendant maximum, not simultaneous process-tree RSS or sum",
      artifactSize:
        "actual supplied normal CLI file bytes, not compressed package or shipping artifact size",
      accuracy:
        "bounded authored witnesses; observed matching output alone does not establish absence of false negatives",
      thresholds: "advisory hints only; no gate/budget failure and no confirmation rerun",
    },
    environment: {
      platform: process.platform,
      architecture: arch(),
      kernel: release(),
      node: process.version,
      cpuModel: cpus()[0]?.model ?? null,
      affinity: existsSync("/proc/self/status")
        ? (/^Cpus_allowed_list:\s*(.*)$/mu.exec(readFileSync("/proc/self/status", "utf8"))?.[1] ??
          null)
        : null,
    },
    binaries,
    backend,
    vue,
    corpus,
    corpusAfterSha256: after.sha256,
    identitiesUnchanged: unchanged,
    binarySizeRatio: binaries.base.bytes > 0 ? binaries.head.bytes / binaries.base.bytes : null,
    results,
  };
  writeJson(join(out, "report.json"), report);
  const summary = [
    "## PR performance observation",
    "",
    `Base: [${options.baseSource.slice(0, 12)}](https://github.com/ubugeeei-prod/vize/commit/${options.baseSource}) · Head: [${options.headSource.slice(0, 12)}](https://github.com/ubugeeei-prod/vize/commit/${options.headSource})`,
    "",
    "Advisory only. Fresh processes with warmups; persistent caches may exist. This is not the cold 500 ms qualification.",
    `${results.filter((row) => row.comparison.availability === "observed").length} observed lanes; ${results.filter((row) => row.comparison.availability === "unavailable").length} unavailable lanes.`,
    "",
    "| Lane | Base ms | Head ms | Paired ratio | Max RSS base/head | Observation |",
    "| --- | ---: | ---: | ---: | ---: | --- |",
    ...results.map((row) => {
      const result: BenchmarkResult | null = row.comparison.result;
      const status = result?.status === "regression" ? "regression candidate" : result?.status;
      return `| ${row.label} | ${result?.baseMs.toFixed(2) ?? "n/a"} | ${result?.headMs.toFixed(2) ?? "n/a"} | ${result?.rate.toFixed(3) ?? "n/a"} | ${row.rss.base ?? "n/a"} / ${row.rss.head ?? "n/a"} B | ${row.reason?.replaceAll("|", "/") ?? status ?? "unavailable"} |`;
    }),
    "",
    "RSS is one separate GNU time sample: maximum individual process/waited child, not the sum of simultaneous Rust/Go memory.",
    "",
    `Normal CLI bytes: ${binaries.base.bytes} → ${binaries.head.bytes}. Full hashes, versions, corpus identities, independent controls and raw commands are in the artifact.`,
    "",
  ];
  writeFileSync(join(out, "summary.md"), summary.join("\n"));
  return report;
}
