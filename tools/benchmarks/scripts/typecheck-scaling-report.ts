/**
 * Medians, per-SFC cost and superlinearity verdicts of the type-check scaling
 * lane, and its Markdown summary. Pure: reads the results the driver wrote.
 *
 * A tool is flagged when its per-SFC CPU or peak RSS at the largest size
 * exceeds GROWTH_LIMIT × its value at the smallest size, or when the marginal
 * cost per added SFC between the two largest sizes exceeds GROWTH_LIMIT × the
 * marginal cost between the two smallest (a step in the curve).
 */

import { PLANT_MULTIPLICITY_NOTES } from "./scale-project-plants.ts";
import type { ScaleKind } from "./scale-project-generator.ts";
import type { Provenance, ToolId } from "./typecheck-scaling-tools.ts";

export const GROWTH_LIMIT = 1.5;
export const MAPPER_SPEEDUP_TARGET = 100;
export const MAPPER_MEMORY_TARGET = 10;
export const TSRS_WALL_TARGET_MS = 500;

export type Sample = {
  wallMs: number;
  cpuMs: number | null;
  maxRssKb: number | null;
  errors: number;
  warnings: number;
  fingerprint: string;
  types: number | null;
  instantiations: number | null;
  checkTimeS: number | null;
};
export type Cell = {
  tool: ToolId;
  label: string;
  vize: boolean;
  kind: ScaleKind;
  sfcs: number;
  plants: number;
  mode: string;
  cold: Sample | null;
  runs: Sample[];
  refused: string | null;
};
export type ScalingResults = {
  schemaVersion: 1;
  sha: string;
  runner: string;
  machine: string;
  measured: boolean;
  runs: number;
  generatedAt: string;
  runUrl: string | null;
  nodeOptions: string;
  gnuTime: string | null;
  provenance: Provenance;
  cells: Cell[];
};

export function median(values: number[]): number {
  if (values.length === 0) throw new Error("median of no values");
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1
    ? (sorted[mid] as number)
    : ((sorted[mid - 1] as number) + (sorted[mid] as number)) / 2;
}

const medianOf = (samples: Sample[], key: keyof Sample): number | null => {
  const values = samples.map((sample) => sample[key]).filter((v) => typeof v === "number");
  return values.length === samples.length && values.length > 0 ? median(values as number[]) : null;
};

export type Row = {
  cell: Cell;
  wallMs: number | null;
  wallDeviationMs: number | null;
  cpuMs: number | null;
  cpuDeviationMs: number | null;
  rssKb: number | null;
  rssDeviationKb: number | null;
  types: number | null;
  instantiations: number | null;
  checkTimeS: number | null;
};

export function sampleDeviation(values: number[]): number {
  if (values.length < 2) return 0;
  const mean = values.reduce((sum, value) => sum + value, 0) / values.length;
  return Math.sqrt(
    values.reduce((sum, value) => sum + (value - mean) ** 2, 0) / (values.length - 1),
  );
}

export type PerformanceTarget = {
  kind: ScaleKind;
  sfcs: number;
  diagnosticParity: boolean;
  mapperWarmSpeedup: number | null;
  mapperColdSpeedup: number | null;
  mapperWarmMemoryReduction: number | null;
  mapperColdMemoryReduction: number | null;
  tsrsWarmMs: number | null;
  tsrsColdMs: number | null;
  met: boolean;
  missing: string[];
};

/** Warm medians; the cold run never enters them. A refused cell has no numbers. */
export function rowOf(cell: Cell): Row {
  const runs = cell.refused ? [] : cell.runs;
  const of = (key: keyof Sample) => (runs.length > 0 ? medianOf(runs, key) : null);
  const deviationOf = (key: keyof Sample) => {
    const values = runs.map((sample) => sample[key]).filter((value) => typeof value === "number");
    return values.length === runs.length && values.length > 0
      ? sampleDeviation(values as number[])
      : null;
  };
  return {
    cell,
    wallMs: of("wallMs"),
    wallDeviationMs: deviationOf("wallMs"),
    cpuMs: of("cpuMs"),
    cpuDeviationMs: deviationOf("cpuMs"),
    rssKb: of("maxRssKb"),
    rssDeviationKb: deviationOf("maxRssKb"),
    types: of("types"),
    instantiations: of("instantiations"),
    checkTimeS: of("checkTimeS"),
  };
}

const sampleFingerprint = (cell: Cell | undefined): string | null =>
  cell != null && cell.refused == null && cell.runs.length > 0
    ? (cell.runs[0]?.fingerprint ?? null)
    : null;

/**
 * The public performance targets compare the same host on the same corpus.
 * A target is never eligible when either lane was refused or their complete
 * diagnostic fingerprints differ.
 */
export function performanceTargets(cells: Cell[]): PerformanceTarget[] {
  const keys = [
    ...new Set(
      cells
        .filter((cell) => cell.tool === "tsc-vize-mapper")
        .map((cell) => `${cell.kind}:${cell.sfcs}`),
    ),
  ];
  return keys.map((key) => {
    const [kind, sfcsText] = key.split(":") as [ScaleKind, string];
    const sfcs = Number(sfcsText);
    const find = (tool: Cell["tool"]) =>
      cells.find((cell) => cell.kind === kind && cell.sfcs === sfcs && cell.tool === tool);
    const vize = find("tsc-vize-mapper");
    const reference = find("tsc-vue-mapper");
    const tsrs = find("tsrs-fast-parallel-vize-mapper");
    const vizeRow = vize ? rowOf(vize) : null;
    const referenceRow = reference ? rowOf(reference) : null;
    const tsrsRow = tsrs ? rowOf(tsrs) : null;
    const divide = (
      numerator: number | null | undefined,
      denominator: number | null | undefined,
    ) =>
      numerator != null && denominator != null && denominator > 0 ? numerator / denominator : null;
    const diagnosticParity =
      sampleFingerprint(vize) != null &&
      sampleFingerprint(vize) === sampleFingerprint(reference) &&
      sampleFingerprint(vize) === sampleFingerprint(tsrs);
    const mapperWarmSpeedup = divide(referenceRow?.wallMs, vizeRow?.wallMs);
    const mapperColdSpeedup = divide(reference?.cold?.wallMs, vize?.cold?.wallMs);
    const mapperWarmMemoryReduction = divide(referenceRow?.rssKb, vizeRow?.rssKb);
    const mapperColdMemoryReduction = divide(reference?.cold?.maxRssKb, vize?.cold?.maxRssKb);
    const tsrsWarmMs = tsrsRow?.wallMs ?? null;
    const tsrsColdMs = tsrs?.cold?.wallMs ?? null;
    const missing: string[] = [];
    if (!diagnosticParity) missing.push("diagnostic parity");
    if ((mapperWarmSpeedup ?? 0) < MAPPER_SPEEDUP_TARGET) missing.push("warm mapper 100x");
    if ((mapperColdSpeedup ?? 0) < MAPPER_SPEEDUP_TARGET) missing.push("cold mapper 100x");
    if ((mapperWarmMemoryReduction ?? 0) < MAPPER_MEMORY_TARGET) missing.push("warm memory 10x");
    if ((mapperColdMemoryReduction ?? 0) < MAPPER_MEMORY_TARGET) missing.push("cold memory 10x");
    if ((tsrsWarmMs ?? Number.POSITIVE_INFINITY) > TSRS_WALL_TARGET_MS)
      missing.push("warm tsrs 500ms");
    if ((tsrsColdMs ?? Number.POSITIVE_INFINITY) > TSRS_WALL_TARGET_MS)
      missing.push("cold tsrs 500ms");
    return {
      kind,
      sfcs,
      diagnosticParity,
      mapperWarmSpeedup,
      mapperColdSpeedup,
      mapperWarmMemoryReduction,
      mapperColdMemoryReduction,
      tsrsWarmMs,
      tsrsColdMs,
      met: missing.length === 0,
      missing,
    };
  });
}

type Metric = "wallMs" | "cpuMs" | "rssKb";
export type Growth = {
  tool: ToolId;
  kind: ScaleKind;
  label: string;
  vize: boolean;
  sizes: number[];
  /** Per-SFC value at the largest size over the smallest. */
  perSfc: Record<Metric, number | null>;
  /** Marginal cost per added SFC, last interval over the first; null with fewer than three sizes. */
  step: Record<Metric, number | null>;
  flags: string[];
};

const ratio = (a: number | null, b: number | null): number | null =>
  a == null || b == null || !(b > 0) ? null : a / b;

/** Growth per tool and project kind (vue, nuxt) over the sizes that measured. */
export function growthOf(rows: Row[]): Growth[] {
  const pairs = [...new Set(rows.map((row) => `${row.cell.kind}:${row.cell.tool}`))];
  return pairs.flatMap((pair) => {
    const [kind, tool] = pair.split(":") as [ScaleKind, ToolId];
    if (kind === "monorepo") return [];
    const series = rows
      .filter((row) => row.cell.tool === tool && row.cell.kind === kind && row.cpuMs != null)
      .sort((a, b) => a.cell.sfcs - b.cell.sfcs);
    const first = series[0];
    const last = series.at(-1);
    if (!first || !last || series.length < 2) return [];
    const perSfc = (metric: Metric) =>
      ratio(
        last[metric] == null ? null : (last[metric] as number) / last.cell.sfcs,
        first[metric] == null ? null : (first[metric] as number) / first.cell.sfcs,
      );
    const slope = (a: Row, b: Row, metric: Metric) =>
      a[metric] == null || b[metric] == null
        ? null
        : ((b[metric] as number) - (a[metric] as number)) / (b.cell.sfcs - a.cell.sfcs);
    const step = (metric: Metric) => {
      if (series.length < 3) return null;
      const [a, b] = series as [Row, Row];
      const [y, z] = series.slice(-2) as [Row, Row];
      return ratio(slope(y, z, metric), slope(a, b, metric));
    };
    const growth: Growth = {
      tool,
      kind,
      label: kind === "nuxt" ? `${first.cell.label} (Nuxt)` : first.cell.label,
      vize: first.cell.vize,
      sizes: series.map((row) => row.cell.sfcs),
      perSfc: { wallMs: perSfc("wallMs"), cpuMs: perSfc("cpuMs"), rssKb: perSfc("rssKb") },
      step: { wallMs: step("wallMs"), cpuMs: step("cpuMs"), rssKb: step("rssKb") },
      flags: [],
    };
    for (const [metric, name] of [
      ["cpuMs", "CPU"],
      ["rssKb", "RSS"],
    ] as const) {
      const value = growth.perSfc[metric];
      if (value != null && value > GROWTH_LIMIT) {
        growth.flags.push(
          `per-SFC ${name} ×${value.toFixed(2)} from ${first.cell.sfcs} to ${last.cell.sfcs}`,
        );
      }
      const jump = growth.step[metric];
      if (jump != null && jump > GROWTH_LIMIT) {
        growth.flags.push(`marginal ${name} per SFC steps ×${jump.toFixed(2)}`);
      }
    }
    return [growth];
  });
}

const fixed = (value: number | null, digits: number, scale = 1) =>
  value == null ? "n/a" : (value / scale).toFixed(digits);
const count = (value: number | null) =>
  value == null ? "n/a" : Math.round(value).toLocaleString("en-US");
const times = (value: number | null) => (value == null ? "n/a" : `×${value.toFixed(2)}`);

export function renderSummary(results: ScalingResults): string {
  const rows = results.cells.map(rowOf);
  const growth = growthOf(rows);
  const targets = performanceTargets(results.cells);
  const refused = results.cells.filter((cell) => cell.refused);
  const lines = [
    "# Type-check scaling",
    "",
    `Commit \`${results.sha}\` on \`${results.runner}\` (${results.machine}), ${results.generatedAt}. Median and sample standard deviation (σ) of ${results.runs} fresh-process run(s) after the first (cold) run; the cold run is listed separately and never enters either statistic.${results.runUrl ? ` [Run](${results.runUrl}).` : ""}`,
    "",
    "CPU is user+sys of the whole process tree (reaped children's cutime+cstime from /proc/self/stat around each spawn). Peak RSS is GNU time `%M`, the largest single process of the tree. Every timed run reported exactly the planted errors; each run starts with an empty cache directory (`XDG_CACHE_HOME`), so no checker replays an earlier run's results, and build-mode runs start from deleted outputs with `--force`. These samples do not represent a persistent-session warm unchanged request; that lifecycle is measured by the LSP residency pair.",
    "",
  ];
  for (const note of PLANT_MULTIPLICITY_NOTES) lines.push(`- ${note}`);
  if (PLANT_MULTIPLICITY_NOTES.length > 0) lines.push("");
  if (!results.measured) lines.push("**Smoke run: not on Linux, so nothing was timed.**", "");
  if (refused.length > 0) {
    lines.push(
      `## Refused (${refused.length})`,
      "",
      "These cells did not report exactly the planted errors, so they have no timing.",
      "",
    );
    for (const cell of refused) {
      lines.push(
        `- **${cell.label}**, ${cell.kind} ${cell.sfcs} SFCs:`,
        "",
        "```diagnostic",
        cell.refused ?? "",
        "```",
        "",
      );
    }
  }
  const header = (kind: string) => [
    `| Tool | ${kind === "monorepo" ? "Mode" : "SFCs"} | Wall s | Wall σ s | CPU s | CPU σ s | Peak RSS MiB | RSS σ MiB | Wall ms/SFC | CPU ms/SFC | RSS KiB/SFC | Types | Instantiations | Check s | Cold wall s | Cold CPU s |`,
    "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
  ];
  const line = (row: Row) => {
    const { cell } = row;
    const per = (value: number | null, digits: number) =>
      value == null ? "n/a" : (value / cell.sfcs).toFixed(digits);
    return `| ${cell.label} | ${cell.kind === "monorepo" ? `${cell.mode}, ${cell.sfcs}` : cell.sfcs} | ${fixed(row.wallMs, 2, 1000)} | ${fixed(row.wallDeviationMs, 2, 1000)} | ${fixed(row.cpuMs, 2, 1000)} | ${fixed(row.cpuDeviationMs, 2, 1000)} | ${fixed(row.rssKb, 0, 1024)} | ${fixed(row.rssDeviationKb, 0, 1024)} | ${per(row.wallMs, 3)} | ${per(row.cpuMs, 3)} | ${per(row.rssKb, 1)} | ${count(row.types)} | ${count(row.instantiations)} | ${fixed(row.checkTimeS, 2)} | ${cell.refused ? "refused" : fixed(cell.cold?.wallMs ?? null, 2, 1000)} | ${cell.refused ? "refused" : fixed(cell.cold?.cpuMs ?? null, 2, 1000)} |`;
  };
  for (const [kind, title] of [
    ["vue", "Single project"],
    ["nuxt", "Nuxt app"],
  ] as const) {
    const project = rows.filter((row) => row.cell.kind === kind);
    if (project.length === 0) continue;
    lines.push(`## ${title}`, "", ...header(kind));
    for (const row of [...project].sort(
      (a, b) => a.cell.tool.localeCompare(b.cell.tool) || a.cell.sfcs - b.cell.sfcs,
    )) {
      lines.push(line(row));
    }
    lines.push("");
  }
  const monorepo = rows.filter((row) => row.cell.kind === "monorepo");
  if (monorepo.length > 0) {
    lines.push(
      "## Project-references monorepo",
      "",
      ...header("monorepo"),
      ...monorepo.map(line),
      "",
    );
  }
  if (targets.length > 0) {
    lines.push(
      "## Performance targets",
      "",
      `The target is at least ${MAPPER_SPEEDUP_TARGET}x lower mapper wall time and ${MAPPER_MEMORY_TARGET}x lower peak memory than the reference mapper on the same TypeScript host, plus tsrs --fast --checkers=max at or below ${TSRS_WALL_TARGET_MS} ms. Cold and warm observations must both pass, and complete diagnostic fingerprints must match. A miss remains visible and is never averaged with another corpus.`,
      "",
      "| Corpus | Diagnostics | Mapper wall warm / cold | Memory warm / cold | tsrs warm / cold | Verdict |",
      "| --- | --- | ---: | ---: | ---: | --- |",
      ...targets.map((target) => {
        const corpus = `${target.kind} ${target.sfcs.toLocaleString("en-US")} SFCs`;
        const mapper = `${times(target.mapperWarmSpeedup)} / ${times(target.mapperColdSpeedup)}`;
        const memory = `${times(target.mapperWarmMemoryReduction)} / ${times(target.mapperColdMemoryReduction)}`;
        const tsrs = `${fixed(target.tsrsWarmMs, 0)} ms / ${fixed(target.tsrsColdMs, 0)} ms`;
        return `| ${corpus} | ${target.diagnosticParity ? "exact" : "**mismatch / refused**"} | ${mapper} | ${memory} | ${tsrs} | ${target.met ? "**met**" : `miss: ${target.missing.join(", ")}`} |`;
      }),
      "",
    );
  }
  if (growth.length > 0) {
    lines.push(
      "## Growth",
      "",
      `Per-SFC cost at the largest size over the smallest, and the marginal cost per added SFC of the last interval over the first. Above ×${GROWTH_LIMIT} is flagged.`,
      "",
      "| Tool | Sizes | CPU/SFC | RSS/SFC | Wall/SFC | CPU step | RSS step | Verdict |",
      "| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |",
      ...growth.map(
        (g) =>
          `| ${g.label} | ${g.sizes.join(", ")} | ${times(g.perSfc.cpuMs)} | ${times(g.perSfc.rssKb)} | ${times(g.perSfc.wallMs)} | ${times(g.step.cpuMs)} | ${times(g.step.rssKb)} | ${g.flags.length > 0 ? `**superlinear**: ${g.flags.join("; ")}` : "linear"} |`,
      ),
      "",
    );
  }
  lines.push(
    "<details><summary>Tool versions and binary checksums</summary>",
    "",
    "| Tool | Version | Binary SHA-256 |",
    "| --- | --- | --- |",
    ...Object.entries(results.provenance).map(
      ([name, entry]) =>
        `| ${name} | ${entry.version ?? "n/a"} | ${entry.sha256 ? `\`${entry.sha256}\`` : "n/a"} |`,
    ),
    "",
    `NODE_OPTIONS=\`${results.nodeOptions}\`; ${results.gnuTime ?? "no GNU time"}.`,
    "",
    "</details>",
    "",
  );
  return `${lines.join("\n")}\n`;
}
