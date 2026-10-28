import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import type { DiagnosticEvidence, OperationSample } from "./dual-lsp-oracles.ts";
import type { ProductKind } from "./dual-lsp-product.ts";

export type Distribution = {
  samples: number;
  min: number;
  p50: number;
  p95: number;
  p99: number;
  max: number;
};

export type MeasuredSample = OperationSample & {
  revisionReceiptSha256: string;
  startupMs?: number;
  diagnosticV1QualificationMs?: number;
  diagnosticV2QualificationMs?: number;
};

export type SampleOrderReceipt = {
  sampleIndex: number;
  order: readonly ProductKind[];
};

export function summarizeScheduleReceipt(
  rows: readonly SampleOrderReceipt[],
): Record<string, unknown> {
  let vizeFirst = 0;
  let vueFirst = 0;
  rows.forEach((row, index) => {
    assert.equal(row.sampleIndex, index, "schedule sample indices must be contiguous");
    const expectedOrder: readonly ProductKind[] =
      index % 2 === 0 ? ["vize", "vue-language-tools"] : ["vue-language-tools", "vize"];
    assert.deepEqual(
      row.order,
      expectedOrder,
      `sample ${index} must preserve the ABBA product order`,
    );
    if (row.order[0] === "vize") vizeFirst += 1;
    else vueFirst += 1;
  });
  return {
    samples: rows.length,
    vizeFirst,
    vueFirst,
    rows,
    sha256: createHash("sha256").update(JSON.stringify(rows)).digest("hex"),
  };
}

export function assertRevisionReceiptReachability(
  samples: readonly MeasuredSample[],
  receiptIds: readonly string[],
): Record<string, number> {
  const available = new Set(receiptIds);
  assert.ok(available.size > 0, "at least one revision receipt is required");
  for (const [sampleIndex, sample] of samples.entries()) {
    assert.match(sample.revisionReceiptSha256, /^[0-9a-f]{64}$/u);
    assert.ok(
      available.has(sample.revisionReceiptSha256),
      `sample ${sampleIndex} cannot reach its revision receipt`,
    );
  }
  return {
    samples: samples.length,
    availableReceipts: available.size,
    referencedReceipts: new Set(samples.map((sample) => sample.revisionReceiptSha256)).size,
  };
}

export function distribution(values: readonly number[]): Distribution {
  assert.ok(values.length > 0, "a distribution requires at least one sample");
  const sorted = [...values].sort((left, right) => left - right);
  const quantile = (value: number) => sorted[Math.max(0, Math.ceil(sorted.length * value) - 1)];
  return {
    samples: sorted.length,
    min: sorted[0],
    p50: quantile(0.5),
    p95: quantile(0.95),
    p99: quantile(0.99),
    max: sorted.at(-1)!,
  };
}

export function summarizeSamples(samples: readonly MeasuredSample[]): Record<string, Distribution> {
  const result: Record<string, Distribution> = {
    totalMs: distribution(samples.map((sample) => sample.totalMs)),
    rssKiB: distribution(samples.map((sample) => sample.rss.totalKiB)),
    processCount: distribution(samples.map((sample) => sample.rss.processes)),
  };
  const lanes = new Set(samples.flatMap((sample) => Object.keys(sample.timingsMs)));
  for (const lane of [...lanes].sort()) {
    result[`${lane}Ms`] = distribution(samples.map((sample) => sample.timingsMs[lane]));
  }
  for (const metric of [
    "startupMs",
    "diagnosticV1QualificationMs",
    "diagnosticV2QualificationMs",
  ] as const) {
    if (samples.every((sample) => sample[metric] != null)) {
      result[metric] = distribution(samples.map((sample) => sample[metric]!));
    }
  }
  return result;
}

export function summarizeDiagnostics(
  samples: ReadonlyArray<{ v1: DiagnosticEvidence; v2: DiagnosticEvidence }>,
): Record<string, Distribution> {
  return {
    v1PublicationLatencyMs: distribution(samples.map((sample) => sample.v1.publicationLatencyMs)),
    v1QualificationMs: distribution(samples.map((sample) => sample.v1.qualificationMs)),
    v2PublicationLatencyMs: distribution(samples.map((sample) => sample.v2.publicationLatencyMs)),
    v2QualificationMs: distribution(samples.map((sample) => sample.v2.qualificationMs)),
  };
}

export function writeDualLspReport(outputDir: string, report: Record<string, unknown>): void {
  fs.mkdirSync(outputDir, { recursive: true });
  fs.writeFileSync(path.join(outputDir, "report.json"), `${JSON.stringify(report, null, 2)}\n`);
  fs.writeFileSync(path.join(outputDir, "summary.md"), renderSummary(report));
}

function renderFailure(value: unknown): string {
  if (typeof value === "string") return value;
  if (value == null) return "unknown";
  try {
    return JSON.stringify(value) ?? "unknown";
  } catch {
    return "unserializable failure";
  }
}

function renderSummary(report: Record<string, unknown>): string {
  if (report.status !== "passed") {
    return [
      "# Same-wire LSP benchmark",
      "",
      "Status: **failed**",
      "",
      `Failure: ${renderFailure(report.failure)}`,
      "",
      "No failed, stale, incomplete, or extra result is admitted as a performance sample.",
      "",
    ].join("\n");
  }
  const products = report.products as Record<
    ProductKind,
    { freshSummary: Record<string, Distribution>; warmSummary: Record<string, Distribution> }
  >;
  const rows: string[] = [];
  for (const phase of ["fresh", "warm"] as const) {
    for (const product of ["vize", "vue-language-tools"] as const) {
      const summary =
        phase === "fresh" ? products[product].freshSummary : products[product].warmSummary;
      for (const [metric, stats] of Object.entries(summary)) {
        rows.push(
          `| ${phase} | ${product} | ${metric} | ${stats.samples} | ${stats.p50.toFixed(2)} | ${stats.p95.toFixed(2)} | ${stats.p99.toFixed(2)} |`,
        );
      }
    }
  }
  return [
    "# Same-wire LSP benchmark",
    "",
    "Status: **passed**",
    "",
    "Only correctness-qualified samples are present. Fresh means a new complete product process tree; warm means repeated operations on one initialized product. startupMs ends after initialize, diagnostic qualification metrics end after product-wide barriers and quiescence, and totalMs contains only the six query operations. Vue Language Tools includes both Vue LS and TypeScript LS process trees.",
    "",
    "`PublishDiagnosticsParams.version` is absent from TypeScript Language Server. The Vue row therefore records `reportedVersion: null` and `provenVersion: 2` after mutually exclusive content fingerprints, a post-change barrier, and quiescence. This is not version-field parity.",
    "",
    "| Phase | Product | Metric | n | p50 | p95 | p99 |",
    "| --- | --- | --- | ---: | ---: | ---: | ---: |",
    ...rows,
    "",
    "Rust allocation/function profiles and Node CPU profiles are attribution artifacts only; they are not compared as the same metric.",
    "",
  ].join("\n");
}
