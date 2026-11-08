#!/usr/bin/env node

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface ProcessRow {
  readonly executable?: string;
  readonly executables?: readonly string[];
  readonly cpu_seconds: number;
  readonly peak_rss_bytes?: number;
  readonly rss_bytes?: number;
}

interface ControlledEdit {
  readonly edit: string;
  readonly elapsed_ms: number;
}

interface RecoveryPhase {
  readonly final_diagnostics?: unknown;
  readonly final_importer_diagnostics?: unknown;
  readonly completion?: { readonly label?: unknown; readonly detail?: unknown };
  readonly hover?: { readonly text?: unknown };
}

interface MeasurementRun {
  readonly corpus: unknown;
  readonly opened_files: number;
  readonly metadata_requests: number;
  readonly successful_unique_provider_requests: number;
  readonly resident_document_witnesses: number;
  readonly controlled_edits: readonly ControlledEdit[];
  readonly cancellation_churn?: {
    readonly recovery?: { readonly before?: RecoveryPhase; readonly after?: RecoveryPhase };
    readonly diagnostic_publications?: {
      readonly tracked_uris?: unknown;
      readonly nondecreasing_versions_per_uri?: unknown;
      readonly known_document_source_versions?: unknown;
      readonly all_observed_uri_tails_clean?: unknown;
      readonly final_child_revision_clean?: unknown;
      readonly publications_after_final_fence?: unknown;
      readonly tails?: readonly {
        readonly uri?: unknown;
        readonly version?: unknown;
        readonly diagnostics?: unknown;
        readonly source_sha256?: unknown;
      }[];
    };
    readonly diagnostic_fences?: {
      readonly no_publications_after_either_clean_fence?: unknown;
    };
    readonly retention?: { readonly cohort_identical?: unknown };
  };
  readonly elapsed_ms: number;
  readonly initialized_ms: number;
  readonly warming_elapsed_ms: number;
  readonly observed_processes: readonly ProcessRow[];
  readonly idle_processes: readonly ProcessRow[];
  readonly rss_peak_mib: number;
  readonly rss_idle_mib: number;
  readonly profile_json?: {
    readonly path?: string;
    readonly bytes?: number;
    readonly sha256?: string;
  };
  readonly error?: unknown;
  readonly finalization_error?: unknown;
}

interface Measurement {
  readonly complete: boolean;
  readonly completed_runs: number;
  readonly workspace_files: number;
  readonly open_files: number;
  readonly unique_imported_providers: number;
  readonly server: string;
  readonly per_run: readonly MeasurementRun[];
}

interface AllocationProfile {
  readonly command: string;
  readonly allocation?: {
    readonly calls: number;
    readonly requested_bytes: number;
    readonly released_bytes: number;
  };
}

interface MetricRow {
  readonly wall_ms: number;
  readonly cpu_seconds: number;
  readonly initialization_ms: number;
  readonly warm_residency_ms: number;
  readonly edit_ms: number;
  readonly server_peak_mib: number;
  readonly server_idle_mib: number;
  readonly process_tree_peak_mib: number;
  readonly process_tree_idle_mib: number;
  readonly allocation_calls: number;
  readonly allocation_requested_bytes: number;
  readonly allocation_released_bytes: number;
}

type Role = "base" | "head";
type Metric = keyof MetricRow;
type MetricSummary = Readonly<{
  median: number;
  sample_variance: number;
  values: readonly number[];
}>;
type RoleSummary = Readonly<Record<Metric, MetricSummary>>;

const roleNames = ["base", "head"] as const;

const mib = (bytes: number): number => bytes / 1024 / 1024;

const requiredNumber = (value: number | undefined, label: string): number => {
  if (typeof value !== "number" || !Number.isFinite(value)) throw new TypeError(label);
  return value;
};

const median = (values: readonly number[]): number => {
  assert.ok(values.length > 0, "median requires samples");
  const sorted = values.toSorted((left, right) => left - right);
  return sorted[Math.floor(sorted.length / 2)];
};

const sampleVariance = (values: readonly number[]): number => {
  assert.ok(values.length > 1, "sample variance requires multiple samples");
  const mean = values.reduce((total, value) => total + value, 0) / values.length;
  return values.reduce((total, value) => total + (value - mean) ** 2, 0) / (values.length - 1);
};

const normalizeWorkspace = (value: unknown): unknown => {
  if (typeof value === "string") {
    return value
      .replaceAll(/file:\/\/[^\s"']*\/src\//gu, "file://<workspace>/src/")
      .replaceAll(/\/tmp\/vize-real-lsp-resource-[^/\s"']+/gu, "<workspace>");
  }
  if (Array.isArray(value)) return value.map(normalizeWorkspace);
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .filter(([key]) => !["elapsed_ms", "monotonic_ns"].includes(key))
        .map(([key, entry]) => [key, normalizeWorkspace(entry)]),
    );
  }
  return value;
};

const recoveryReceipt = (
  recovery?: Readonly<{ before?: RecoveryPhase; after?: RecoveryPhase }>,
): unknown =>
  Object.fromEntries(
    (["before", "after"] as const).map((phase) => {
      const row = recovery?.[phase];
      return [
        phase,
        normalizeWorkspace({
          final_diagnostics: row?.final_diagnostics,
          final_importer_diagnostics: row?.final_importer_diagnostics,
          completion: { label: row?.completion?.label, detail: row?.completion?.detail },
          hover: { text: row?.hover?.text },
        }),
      ];
    }),
  );

const publicationReceipt = (
  audit?: MeasurementRun["cancellation_churn"] extends infer Churn
    ? Churn extends { readonly diagnostic_publications?: infer Audit }
      ? Audit
      : never
    : never,
): unknown => ({
  tracked_uris: normalizeWorkspace(audit?.tracked_uris),
  nondecreasing_versions_per_uri: audit?.nondecreasing_versions_per_uri,
  known_document_source_versions: audit?.known_document_source_versions,
  all_observed_uri_tails_clean: audit?.all_observed_uri_tails_clean,
  final_child_revision_clean: audit?.final_child_revision_clean,
  publications_after_final_fence: audit?.publications_after_final_fence,
  tails: normalizeWorkspace(
    audit?.tails?.map(({ uri, version, diagnostics, source_sha256 }) => ({
      uri,
      version,
      diagnostics,
      source_sha256,
    })),
  ),
});

const semanticReceipt = (run: MeasurementRun): unknown => ({
  corpus: run.corpus,
  opened_files: run.opened_files,
  metadata_requests: run.metadata_requests,
  successful_unique_provider_requests: run.successful_unique_provider_requests,
  resident_document_witnesses: run.resident_document_witnesses,
  controlled_edits: normalizeWorkspace(run.controlled_edits),
  recovery: recoveryReceipt(run.cancellation_churn?.recovery),
  diagnostic_publications: publicationReceipt(run.cancellation_churn?.diagnostic_publications),
  no_publications_after_clean_fences:
    run.cancellation_churn?.diagnostic_fences?.no_publications_after_either_clean_fence,
});

const processFor = (rows: readonly ProcessRow[], server: string): ProcessRow => {
  const resolved = path.resolve(server);
  const matches = rows.filter((row) =>
    [...(row.executable == null ? [] : [row.executable]), ...(row.executables ?? [])].some(
      (executable) => path.resolve(executable) === resolved,
    ),
  );
  assert.equal(matches.length, 1, "one exact Maestro server process is required");
  return matches[0]!;
};

const metricRow = (measurement: Measurement, profile: AllocationProfile): MetricRow => {
  const run = measurement.per_run[0]!;
  const serverPeak = processFor(run.observed_processes, measurement.server);
  const serverIdle = processFor(run.idle_processes, measurement.server);
  const edit = run.controlled_edits.find((entry) => entry.edit === "body change");
  assert.ok(edit, "controlled edit timing is required");
  assert.ok(Number.isFinite(edit.elapsed_ms), "controlled edit timing is required");
  const serverPeakRss = requiredNumber(serverPeak.peak_rss_bytes, "server peak RSS is required");
  const serverIdleRss = requiredNumber(serverIdle.rss_bytes, "server idle RSS is required");
  const allocation = profile.allocation;
  assert.ok(allocation && allocation.calls > 0, "allocation profile is required");
  return {
    wall_ms: run.elapsed_ms,
    cpu_seconds: run.observed_processes.reduce((total, row) => total + row.cpu_seconds, 0),
    initialization_ms: run.initialized_ms,
    warm_residency_ms: run.warming_elapsed_ms,
    edit_ms: edit.elapsed_ms,
    server_peak_mib: mib(serverPeakRss),
    server_idle_mib: mib(serverIdleRss),
    process_tree_peak_mib: run.rss_peak_mib,
    process_tree_idle_mib: run.rss_idle_mib,
    allocation_calls: allocation.calls,
    allocation_requested_bytes: allocation.requested_bytes,
    allocation_released_bytes: allocation.released_bytes,
  };
};

const readRole = (
  root: string,
  sample: number,
  role: Role,
): Readonly<{ measurement: Measurement; metric: MetricRow; semantic: unknown }> => {
  const measurementPath = path.join(root, `sample-${sample}-${role}.json`);
  const measurement = JSON.parse(fs.readFileSync(measurementPath, "utf8")) as Measurement;
  assert.equal(measurement.complete, true, `${role} sample ${sample} must complete`);
  assert.equal(measurement.completed_runs, 1);
  assert.equal(measurement.per_run.length, 1);
  assert.equal(measurement.workspace_files, 2500);
  assert.equal(measurement.open_files, 2500);
  assert.equal(measurement.unique_imported_providers, 16);
  const run = measurement.per_run[0]!;
  assert.equal(run.error, undefined);
  assert.equal(run.finalization_error, undefined);
  assert.equal(run.opened_files, 2500);
  assert.equal(run.resident_document_witnesses, 2500);
  assert.equal(run.successful_unique_provider_requests, 16);
  assert.equal(run.cancellation_churn?.retention?.cohort_identical, true);
  assert.equal(
    run.cancellation_churn?.diagnostic_fences?.no_publications_after_either_clean_fence,
    true,
  );
  const profileReceipt = run.profile_json;
  assert.ok(profileReceipt, "allocation profile receipt is required");
  const profileRelative = path.join(`sample-${sample}-${role}-profile`, "run-1.json");
  assert.ok(
    profileReceipt.path?.endsWith(`${path.sep}${profileRelative}`),
    "allocation profile path is bound to this sample",
  );
  const profileBytes = fs.readFileSync(path.join(root, profileRelative));
  assert.equal(profileBytes.byteLength, profileReceipt.bytes, "allocation profile byte receipt");
  assert.equal(
    createHash("sha256").update(profileBytes).digest("hex"),
    profileReceipt.sha256,
    "allocation profile digest receipt",
  );
  const profile = JSON.parse(profileBytes.toString("utf8")) as AllocationProfile;
  assert.equal(profile.command, "lsp");
  return { measurement, metric: metricRow(measurement, profile), semantic: semanticReceipt(run) };
};

export function summarizeResidencyPair(root: string, baseSha: string, headSha: string) {
  assert.match(baseSha, /^[0-9a-f]{40}$/u);
  assert.match(headSha, /^[0-9a-f]{40}$/u);
  const records: Record<Role, MetricRow[]> = { base: [], head: [] };
  let reference: unknown;
  for (let sample = 1; sample <= 3; sample++) {
    for (const role of roleNames) {
      const current = readRole(root, sample, role);
      reference ??= current.semantic;
      assert.deepEqual(
        current.semantic,
        reference,
        `diagnostics and successful workload must match: ${role}/${sample}`,
      );
      records[role].push(current.metric);
    }
  }
  const metrics = Object.keys(records.base[0]!) as Metric[];
  const roles = Object.fromEntries(
    roleNames.map((role) => [
      role,
      Object.fromEntries(
        metrics.map((metric) => {
          const values = records[role].map((row) => row[metric]);
          return [
            metric,
            { median: median(values), sample_variance: sampleVariance(values), values },
          ];
        }),
      ),
    ]),
  ) as unknown as Readonly<Record<Role, RoleSummary>>;
  const comparison = Object.fromEntries(
    metrics.map((metric) => {
      const base = roles.base[metric].median;
      const head = roles.head[metric].median;
      return [metric, { delta: head - base, ratio: head / base }];
    }),
  ) as Readonly<Record<Metric, Readonly<{ delta: number; ratio: number }>>>;
  return {
    schema: "vize.lsp-residency-pair.v1",
    source: { base: baseSha, head: headSha },
    controls: {
      warmups_discarded_per_role: 1,
      samples_per_role: 3,
      order: ["base/head warmup", "head/base", "base/head", "head/base"],
      runner_scope: "same runner; fresh process and generated workspace for every role/sample",
      workload:
        "2500 open SFCs; 16 warmed providers; completion, hover, edits, cancellation, residency and idle fence",
      diagnostics:
        "exact normalized diagnostics and successful workload receipts required across every sample",
    },
    roles,
    comparison,
  };
}

const markdown = (summary: ReturnType<typeof summarizeResidencyPair>): string => {
  const labels: Readonly<Record<Metric, string>> = {
    wall_ms: "Wall ms",
    cpu_seconds: "Process-tree CPU s",
    initialization_ms: "Initialize ms",
    warm_residency_ms: "2,500 resident ms",
    edit_ms: "Single edit ms",
    server_peak_mib: "Server peak MiB",
    server_idle_mib: "Server idle MiB",
    process_tree_peak_mib: "Tree peak MiB",
    process_tree_idle_mib: "Tree idle MiB",
    allocation_calls: "Rust allocation calls",
    allocation_requested_bytes: "Rust requested bytes",
    allocation_released_bytes: "Rust released bytes",
  };
  const rows = (Object.entries(labels) as [Metric, string][]).map(([key, label]) => {
    const base = summary.roles.base[key];
    const head = summary.roles.head[key];
    const ratio = summary.comparison[key].ratio;
    return `| ${label} | ${base.median.toFixed(2)} | ${base.sample_variance.toFixed(2)} | ${head.median.toFixed(2)} | ${head.sample_variance.toFixed(2)} | ${ratio.toFixed(4)} |`;
  });
  return [
    "### Exact-parent LSP residency pair",
    "",
    `Base \`${summary.source.base}\`; head \`${summary.source.head}\`. One identical 2,500-SFC warmup per role was discarded, then three fresh samples were collected in alternating order. Diagnostics and successful workload receipts are byte-equivalent after workspace-path normalization.`,
    "",
    "| Metric | Base median | Base variance | Head median | Head variance | Head/base |",
    "|---|---:|---:|---:|---:|---:|",
    ...rows,
    "",
  ].join("\n");
};

function argument(name: string): string {
  const index = process.argv.indexOf(name);
  const value = process.argv[index + 1];
  assert.ok(index >= 0 && value, `missing ${name}`);
  return value;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const root = path.resolve(argument("--root"));
  const output = path.resolve(argument("--out"));
  const summary = summarizeResidencyPair(root, argument("--base"), argument("--head"));
  fs.writeFileSync(output, `${JSON.stringify(summary, null, 2)}\n`);
  const report = markdown(summary);
  if (process.env.GITHUB_STEP_SUMMARY) fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, report);
  process.stdout.write(report);
}
